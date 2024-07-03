use std::fs::File;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, json, to_vec, Value};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use zmq::Socket;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Default)]
pub struct Message {
    pub identities: Vec<Vec<u8>>,
    pub header: Value,
    pub parent_header: Value,
    pub metadata: Value,
    pub content: Value,
}

const DELIM: &[u8] = b"<IDS|MSG>";

fn msg_from_parts(parts: Vec<Vec<u8>>, key: &str) -> Message {
    let mut i = parts.split(|part| part.as_slice() == DELIM);
    let identities = i.next().unwrap();
    let raw_msgs = i.next().unwrap();
    if raw_msgs.len() < 4 {
        panic!("Message length");
    }
    let old_mac = hex::decode(&raw_msgs[0]).unwrap();
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).unwrap();
    for msg in &raw_msgs[1..5] {
        mac.update(msg.as_slice());
    }
    mac.verify_slice(&old_mac).expect("Verification failed");
    Message {
        identities: identities.to_owned(),
        header: from_slice(&raw_msgs[1]).unwrap(),
        parent_header: from_slice(&raw_msgs[2]).unwrap(),
        metadata: from_slice(&raw_msgs[3]).unwrap(),
        content: from_slice(&raw_msgs[4]).unwrap(),
    }
}

fn msg_to_parts(msg: &Message, key: &str) -> Vec<Vec<u8>> {
    let mut parts: Vec<Vec<u8>> = msg.identities.clone();
    parts.push(DELIM.to_vec());

    let raw_header = to_vec(&msg.header).unwrap();
    let raw_parent_header = to_vec(&msg.parent_header).unwrap();
    let raw_metadata = to_vec(&msg.metadata).unwrap();
    let raw_content = to_vec(&msg.content).unwrap();
    let raw_msgs = vec![
        raw_header.as_slice(),
        raw_parent_header.as_slice(),
        raw_metadata.as_slice(),
        raw_content.as_slice(),
    ];

    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).unwrap();
    for msg in &raw_msgs {
        mac.update(msg);
    }
    let hmac = hex::encode(mac.finalize().into_bytes().as_slice());
    parts.push(hmac.as_bytes().to_vec());

    for msg in &raw_msgs {
        parts.push(msg.to_vec());
    }

    parts
}

// identities not transferred
pub fn new_msg(msg: &Message, msg_type: &str, content: Value) -> Message {
    let mut header = msg.header.clone();
    header["msg_type"] = json!(msg_type);
    header["username"] = json!("kernel");
    header["msg_id"] = json!(Uuid::new_v4());
    header["date"] = json!(Utc::now());

    Message {
        identities: Vec::new(),
        header,
        parent_header: msg.header.clone(),
        metadata: json!({}),
        content,
    }
}

// identities transferred
pub fn reply_msg(msg: &Message, content: Value) -> Message {
    let old_msg_type = msg.header["msg_type"].as_str().unwrap();
    let msg_type = old_msg_type.to_owned().replace("_request", "_reply");
    let mut reply = new_msg(msg, &msg_type, content);
    reply.identities.clone_from(&msg.identities);
    reply
}

pub fn recv_msg(sock: &Socket, key: &str) -> Message {
    let parts = sock.recv_multipart(0).unwrap();
    msg_from_parts(parts, key)
}

pub fn send_msg(sock: &Socket, key: &str, msg: Message) {
    let parts = msg_to_parts(&msg, key);
    sock.send_multipart(parts, 0).unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct ConnectInfo {
    pub ip: String,
    pub transport: String,
    pub key: String,
    pub signature_scheme: String,
    pub kernel_name: String,
    pub stdin_port: u16,
    pub hb_port: u16,
    pub control_port: u16,
    pub shell_port: u16,
    pub iopub_port: u16,
}

pub fn read_connection_file(filename: String) -> ConnectInfo {
    let f = File::open(filename).unwrap();
    serde_json::from_reader(f).unwrap()
}

pub fn format_address(ci: &ConnectInfo, port: u16) -> String {
    format!(
        "{transport}://{ip}:{port}",
        transport = ci.transport,
        ip = ci.ip
    )
}
