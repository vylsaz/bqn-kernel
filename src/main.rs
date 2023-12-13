mod messaging;
use cbqn_sys::{
    bqn_bound, bqn_call1, bqn_call2, bqn_copy, bqn_directArrType, bqn_eval, bqn_free, bqn_init,
    bqn_makeBoundFn1, bqn_makeBoundFn2, bqn_makeChar, bqn_makeF64, bqn_makeObjVec, bqn_makeUTF8Str,
    bqn_pick, bqn_readC32Arr, bqn_readF64, bqn_type, BQNElType, BQNElType_elt_c16,
    BQNElType_elt_c32, BQNElType_elt_c8, BQNV,
};
use messaging::{
    format_address, msg_from_parts, msg_to_parts, new_msg, read_connection_file, reply_msg,
    ConnectInfo, Message,
};
use serde_json::{json, Value};
use std::{
    ffi::c_char,
    fs::File,
    sync::{Mutex, Once},
    thread,
};
use widestring::U32String;
use zmq::Socket;

fn recv_msg(sock: &Socket, key: &str) -> Message {
    let parts = sock.recv_multipart(0).unwrap();
    msg_from_parts(parts, key)
}

fn send_msg(sock: &Socket, key: &str, msg: Message) {
    let parts = msg_to_parts(&msg, key);
    sock.send_multipart(parts, 0).unwrap();
}

fn kernel_info() -> Value {
    json!({
        "status": "ok",
        "protocol_version": "5.3",
        "implementation": "bqn",
        "implementation_version": "0.1.0",
        "language_info": {
            "name": "BQN",
            "version": "0.1.0",
            "mimetype": "text/bqn",
            "file_extension": ".bqn"
        },
        "banner": "Hello!",
    })
}

static KEY: Mutex<Option<String>> = Mutex::new(None);
static LAST_MSG: Mutex<Option<Message>> = Mutex::new(None);

static IOPUB: Mutex<Option<Socket>> = Mutex::new(None);
static STDIN: Mutex<Option<Socket>> = Mutex::new(None);

fn notebook_input(prompt: &str, password: bool) -> String {
    let key = KEY.lock().unwrap();
    let key = key.as_ref().unwrap();
    let msg = LAST_MSG.lock().unwrap();
    let msg = msg.as_ref().unwrap();
    let stdin = STDIN.lock().unwrap();
    let stdin = stdin.as_ref().unwrap();

    let mut input_req = new_msg(
        msg,
        "input_request",
        json!({
            "prompt": prompt,
            "password": password,
        }),
    );
    input_req.identities = msg.identities.clone();
    send_msg(stdin, key, input_req);

    let input_rep = recv_msg(stdin, key);
    let input = match input_rep.header["msg_type"].as_str() {
        Some("input_reply") => input_rep.content["value"].as_str().unwrap_or(""),
        Some(_) | None => "",
    };
    input.to_owned()
}

fn send_iopub(key: &str, msg: Message) {
    let iopub = IOPUB.lock().unwrap();
    let iopub = iopub.as_ref().unwrap();
    send_msg(iopub, key, msg);
}

fn reply_iopub(msg_type: &str, content: Value) {
    let key = KEY.lock().unwrap();
    let key = key.as_ref().unwrap();
    let msg = LAST_MSG.lock().unwrap();
    let msg = msg.as_ref().unwrap();

    let re = new_msg(msg, msg_type, content);
    send_iopub(key, re);
}

fn notebook_output(name: &str, text: &str) {
    reply_iopub(
        "stream",
        json!({
            "name": name,
            "text": text,
        }),
    );
}

fn notebook_display(mimetype: &str, data: &str) {
    reply_iopub(
        "display_data",
        json!({
            "data": {
                mimetype: data,
            },
            "metadata": {},
        }),
    );
}

fn notebook_output_clear(wait: bool) {
    reply_iopub("clear_output", json!({"wait": wait}));
}

unsafe fn str_to_bqnv(s: &str) -> BQNV {
    bqn_makeUTF8Str(s.len(), s.as_ptr() as *const c_char)
}

// consumes a
unsafe fn str_from_bqnv(a: BQNV) -> String {
    let bound = bqn_bound(a);
    let mut s = Vec::with_capacity(bound);
    s.set_len(bound);
    bqn_readC32Arr(a, s.as_mut_ptr());
    bqn_free(a);
    U32String::from_vec(s).to_string_lossy()
}

// consumes a
unsafe fn f64_from_bqnv(a: BQNV) -> f64 {
    let f = bqn_readF64(a);
    bqn_free(a);
    f
}

unsafe extern "C" fn bqn_notebook_input(obj: BQNV, w: BQNV, x: BQNV) -> BQNV {
    bqn_free(obj);
    let prompt = str_from_bqnv(w);
    let password = f64_from_bqnv(x);
    let result = notebook_input(&prompt, password != 0.0);
    str_to_bqnv(&result)
}

unsafe extern "C" fn bqn_notebook_output(obj: BQNV, w: BQNV, x: BQNV) -> BQNV {
    bqn_free(obj);
    let name = str_from_bqnv(w);
    let text = str_from_bqnv(bqn_copy(x));
    notebook_output(&name, &text);
    x
}

unsafe extern "C" fn bqn_notebook_display(obj: BQNV, w: BQNV, x: BQNV) -> BQNV {
    bqn_free(obj);
    let mimetype = str_from_bqnv(w);
    let data = str_from_bqnv(bqn_copy(x));
    notebook_display(&mimetype, &data);
    x
}

unsafe extern "C" fn bqn_notebook_clear(obj: BQNV, x: BQNV) -> BQNV {
    bqn_free(obj);
    let wait = f64_from_bqnv(x);
    notebook_output_clear(wait != 0.0);
    bqn_makeChar(0)
}

unsafe fn bqn_eval_str(src: &str) -> BQNV {
    let src = str_to_bqnv(src);
    let ret = bqn_eval(src);
    bqn_free(src);
    ret
}

unsafe fn bqn_repl_init() -> BQNV {
    bqn_init();

    let replfun = bqn_eval_str(include_str!("./repl.bqn"));

    let obj = bqn_makeF64(0.0);

    let raw_input = bqn_makeBoundFn2(Some(bqn_notebook_input), obj);
    let raw_output = bqn_makeBoundFn2(Some(bqn_notebook_output), obj);
    let raw_display = bqn_makeBoundFn2(Some(bqn_notebook_display), obj);
    let raw_clear = bqn_makeBoundFn1(Some(bqn_notebook_clear), obj);
    let arg = [raw_input, raw_output, raw_display, raw_clear];
    let replarg = bqn_makeObjVec(arg.len(), arg.as_ptr());

    let repl = bqn_call1(replfun, replarg);

    bqn_free(replarg);
    bqn_free(replfun);
    bqn_free(obj);

    repl
}

static mut TRAP: BQNV = 0;
static INIT: Once = Once::new();

// consumes x
unsafe fn bqn_repl_eval(eval: BQNV, x: BQNV) -> Result<BQNV, String> {
    INIT.call_once(|| {
        TRAP = bqn_eval_str("{1‿(𝕎 𝕩)}⎊{0‿(•CurrentError 𝕩)}");
    });

    let r = bqn_call2(TRAP, eval, x);
    let k = bqn_pick(r, 0);
    let s = bqn_pick(r, 1);
    let ok = f64_from_bqnv(k);
    let ret = if ok != 0.0 {
        Ok(s)
    } else {
        Err(str_from_bqnv(s))
    };
    bqn_free(x);
    bqn_free(r);
    ret
}

// doesn't consume repl
unsafe fn bqn_repl_exec(repl: BQNV, code: &str) -> Result<String, String> {
    let x = bqn_repl_eval(repl, str_to_bqnv(code))?;
    let disp = bqn_eval_str("•fmt");
    let r = bqn_repl_eval(disp, x);
    bqn_free(disp);
    r.map(|o| str_from_bqnv(o))
}

// doesn't consume repl
unsafe fn bqn_repl_exec_with(repl: BQNV, with: &str, code: &str) -> Result<String, String> {
    let with = bqn_repl_eval(repl, str_to_bqnv(with))?;
    let x = bqn_repl_eval(with, str_to_bqnv(code)).map_err(|e| {
        bqn_free(with);
        e
    })?;
    let disp = bqn_repl_eval(with, str_to_bqnv("•fmt")).unwrap_or(bqn_eval_str("•fmt"));
    let r = bqn_repl_eval(disp, x);
    bqn_free(with);
    bqn_free(disp);
    let a = r?;

    if bqn_type(a) != 0 {
        bqn_free(a);
        return Err(format!("•fmt did not return a string"));
    }
    let eltype: BQNElType = bqn_directArrType(a);
    if eltype != BQNElType_elt_c8 && eltype != BQNElType_elt_c16 && eltype != BQNElType_elt_c32 {
        bqn_free(a);
        return Err(format!("•fmt did not return a string"));
    }

    Ok(str_from_bqnv(a))
}

fn shell_execute(key: &str, shell: Socket) {
    let mut execution_count: i32 = 0;

    let repl = unsafe { bqn_repl_init() };

    loop {
        let msg = recv_msg(&shell, key);

        {
            let mut last_msg = LAST_MSG.lock().unwrap();
            *last_msg = Some(msg.clone());
        }

        let busy = new_msg(
            &msg,
            "status",
            json!({
                "execution_state": "busy",
            }),
        );
        let idle = new_msg(
            &msg,
            "status",
            json!({
                "execution_state": "idle",
            }),
        );
        send_iopub(key, busy);

        match msg.header["msg_type"].as_str() {
            Some("kernel_info_request") => {
                let re = reply_msg(&msg, kernel_info());
                send_msg(&shell, key, re);
            }
            Some("execute_request") => {
                execution_count += 1;

                let mut code = msg.content["code"].as_str().unwrap();
                let input = new_msg(
                    &msg,
                    "execute_input",
                    json!({
                        "code": code,
                        "execution_count": execution_count,
                    }),
                );
                send_iopub(key, input);

                let mut silent = msg.content["silent"].as_bool().unwrap_or(false);
                let mut with = None;
                let rslt = loop {
                    if code.starts_with(")") {
                        let (cmd, next) = code.split_once("\n").unwrap_or(("", ""));
                        code = next;

                        if cmd.starts_with(")r") {
                            silent = true;
                        } else if cmd.starts_with(")use") {
                            let (_, r) = cmd.split_once(" ").unwrap_or(("", ""));
                            with = Some(r);
                        } else {
                            break Err(format!("Unknown command {cmd}"));
                        }
                    } else {
                        if let Some(with) = with {
                            break unsafe { bqn_repl_exec_with(repl, with, code) };
                        } else {
                            break unsafe { bqn_repl_exec(repl, code) };
                        }
                    }
                };

                let re = if rslt.is_ok() {
                    let succ = rslt.unwrap();
                    if !silent {
                        let ex_rs = new_msg(
                            &msg,
                            "execute_result",
                            json!({
                                "execution_count": execution_count,
                                "data": {
                                    "text/plain": succ,
                                },
                                "metadata": {},
                            }),
                        );
                        send_iopub(key, ex_rs);
                    }

                    reply_msg(
                        &msg,
                        json!({
                            "status": "ok",
                            "execution_count": execution_count,
                        }),
                    )
                } else {
                    let fail = rslt.unwrap_err();
                    let ex_rs = new_msg(
                        &msg,
                        "error",
                        json!({
                            "ename": "Error",
                            "evalue": fail,
                            "traceback": [fail],
                        }),
                    );
                    send_iopub(key, ex_rs);

                    reply_msg(
                        &msg,
                        json!({
                            "status": "error",
                            "execution_count": execution_count,
                            "ename": "Error",
                            "evalue": fail,
                            "traceback": [fail],
                        }),
                    )
                };
                send_msg(&shell, key, re);
            }
            Some(_) | None => {}
        }
        send_iopub(key, idle);
    }
}

fn run(ci: ConnectInfo) {
    let ctx = zmq::Context::new();

    let hb = ctx.socket(zmq::REP).unwrap();
    hb.bind(&format_address(&ci, ci.hb_port)).unwrap();

    let control = ctx.socket(zmq::ROUTER).unwrap();
    control.bind(&format_address(&ci, ci.control_port)).unwrap();

    let shell = ctx.socket(zmq::ROUTER).unwrap();
    shell.bind(&format_address(&ci, ci.shell_port)).unwrap();

    let stdin = ctx.socket(zmq::ROUTER).unwrap();
    stdin.bind(&format_address(&ci, ci.stdin_port)).unwrap();

    let iopub = ctx.socket(zmq::PUB).unwrap();
    iopub.bind(&format_address(&ci, ci.iopub_port)).unwrap();

    {
        let mut my_stdin = STDIN.lock().unwrap();
        *my_stdin = Some(stdin);
    }
    {
        let mut my_iopub = IOPUB.lock().unwrap();
        *my_iopub = Some(iopub);
    }

    {
        let mut key = KEY.lock().unwrap();
        *key = Some(ci.key.clone());
    }

    thread::spawn(move || loop {
        let heartbeat = hb.recv_bytes(0).unwrap();
        hb.send(&heartbeat, 0).unwrap();
    });

    let shell_key = ci.key.clone();
    thread::spawn(move || {
        shell_execute(&shell_key, shell);
    });

    loop {
        let msg = recv_msg(&control, &ci.key);
        match msg.header["msg_type"].as_str() {
            Some("shutdown_request") => {
                let is_restart = msg.content["restart"].as_bool().unwrap_or(false);
                let re = reply_msg(
                    &msg,
                    json!({
                        "status": "ok",
                        "restart": is_restart,
                    }),
                );
                send_msg(&control, &ci.key, re);
                return;
            }
            Some(_) | None => {}
        }
    }

    // let the OS take care of the other threads...
}

#[cfg(target_os = "windows")]
fn env_list() -> Value {
    let dir = std::env::current_dir().unwrap();
    let dir = dir.to_str().unwrap();
    json!({"PATH": "%PATH%;".to_owned()+dir})
}

#[cfg(not(target_os = "windows"))]
fn env_list() -> Value {
    json!({})
}

fn create_kernel_json() {
    let file = File::create("./bqn/kernel.json").expect("cannot create kernel.json");
    let kernel = json!({
        "argv": [
            std::env::current_exe().unwrap(),
            "-f", "{connection_file}"
        ],
        "display_name": "BQN",
        "language": "BQN",
        "env": env_list(),
    });
    serde_json::to_writer(file, &kernel).expect("cannot write kernel.json");
}

fn main() {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "-f" => {
                let filename = args.next().expect("Missing connection file");
                let ci = read_connection_file(filename);
                return run(ci);
            }
            _ => panic!("Unknown option"),
        }
    } else {
        create_kernel_json();
    }
}
