mod bqn;
mod media;
mod messaging;
use bqn::{BQNValue, BQNV};
use media::{base64_png, base64_wav};
use messaging::{
    format_address, new_msg, read_connection_file, recv_msg, reply_msg, send_msg, Message,
};
use serde_json::{json, Value};
use std::{fs::File, thread};
use zmq::Socket;

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
        "banner": "BQN kernel",
    })
}

struct Context {
    key: String,
    last_msg: Message,
    iopub: Socket,
    stdin: Socket,
}
impl Context {
    fn update_msg(&mut self, new_msg: Message) {
        self.last_msg = new_msg;
    }

    fn send_iopub(&self, msg: Message) {
        send_msg(&self.iopub, &self.key, msg);
    }
    fn reply_iopub(&self, msg_type: &str, content: Value) {
        let msg = &self.last_msg;
        let re = new_msg(msg, msg_type, content);
        self.send_iopub(re);
    }

    fn as_bqnvalue(&mut self) -> BQNValue {
        let ptr = self as *mut Self;
        BQNValue::from(ptr)
    }

    fn from_bqnvalue(value: &BQNValue) -> &Self {
        unsafe {
            let ptr = value.to_ptr::<Context>();
            ptr.as_ref().unwrap()
        }
    }
}

fn notebook_input(ctx: &Context, prompt: &str, password: bool) -> String {
    let key = &ctx.key;
    let msg = &ctx.last_msg;
    let stdin = &ctx.stdin;

    let mut input_req = new_msg(
        msg,
        "input_request",
        json!({
            "prompt": prompt,
            "password": password,
        }),
    );
    input_req.identities.clone_from(&msg.identities);
    send_msg(stdin, key, input_req);

    let input_rep = recv_msg(stdin, key);
    let input = match input_rep.header["msg_type"].as_str() {
        Some("input_reply") => input_rep.content["value"].as_str().unwrap_or(""),
        Some(_) | None => "",
    };
    input.to_owned()
}

fn notebook_output(ctx: &Context, name: &str, text: &str) {
    ctx.reply_iopub(
        "stream",
        json!({
            "name": name,
            "text": text,
        }),
    );
}

fn notebook_display(ctx: &Context, mimetype: &str, data: &str) {
    ctx.reply_iopub(
        "display_data",
        json!({
            "data": {
                mimetype: data,
            },
            "metadata": {},
        }),
    );
}

fn notebook_output_clear(ctx: &Context, wait: bool) {
    ctx.reply_iopub("clear_output", json!({"wait": wait}));
}

unsafe extern "C" fn bqn_notebook_input(obj: BQNV, w: BQNV, x: BQNV) -> BQNV {
    let o = BQNValue::from(obj);
    let w = BQNValue::from(w);
    let x = BQNValue::from(x);
    let ctx = Context::from_bqnvalue(&o);
    let prompt = w.to_string();
    let password = x.to_f64();
    let result = notebook_input(ctx, &prompt, password != 0.0);
    BQNValue::from(result).copy()
}

unsafe extern "C" fn bqn_notebook_output(obj: BQNV, w: BQNV, x: BQNV) -> BQNV {
    let o = BQNValue::from(obj);
    let w = BQNValue::from(w);
    let x = BQNValue::from(x);
    let ctx = Context::from_bqnvalue(&o);
    let name = w.to_string();
    let text = x.to_string();
    notebook_output(ctx, &name, &text);
    x.copy()
}

unsafe extern "C" fn bqn_notebook_display(obj: BQNV, w: BQNV, x: BQNV) -> BQNV {
    let o = BQNValue::from(obj);
    let w = BQNValue::from(w);
    let x = BQNValue::from(x);
    let ctx = Context::from_bqnvalue(&o);
    let mimetype = w.to_string();
    let data = x.to_string();
    notebook_display(ctx, &mimetype, &data);
    x.copy()
}

unsafe extern "C" fn bqn_notebook_clear(obj: BQNV, x: BQNV) -> BQNV {
    let o = BQNValue::from(obj);
    let x = BQNValue::from(x);
    let ctx = Context::from_bqnvalue(&o);
    let wait = x.to_f64();
    notebook_output_clear(ctx, wait != 0.0);
    BQNValue::null().copy()
}

unsafe extern "C" fn bqn_notebook_png(obj: BQNV, x: BQNV) -> BQNV {
    let _ = BQNValue::from(obj);
    let x = BQNValue::from(x);
    // rank has to be = 3
    let shape = x.shape();
    let height = shape[0] as u32;
    let width = shape[1] as u32;
    let channels = shape[2] as u32;

    let img = x.to_f64_vec();
    let data: Vec<u8> = img.iter().map(|&f| (f * 255.0) as u8).collect();
    let png = base64_png(width, height, channels, &data);
    let ret = match png {
        Ok(png) => BQNValue::from([BQNValue::from(1.0), BQNValue::from(png)]),
        Err(err) => BQNValue::from([BQNValue::from(0.0), BQNValue::from(err)]),
    };
    ret.copy()
}

unsafe extern "C" fn bqn_notebook_wav(obj: BQNV, w: BQNV, x: BQNV) -> BQNV {
    let _ = BQNValue::from(obj);
    let w = BQNValue::from(w);
    let x = BQNValue::from(x);
    // rank has to be = 2
    let shape = x.shape();
    let channels = shape[1] as u16;

    let sample_rate = w.to_f64() as u32;

    let aud = x.to_f64_vec();
    let wav = base64_wav(channels, sample_rate, &aud);
    let ret = match wav {
        Ok(wav) => BQNValue::from([BQNValue::from(1.0), BQNValue::from(wav)]),
        Err(err) => BQNValue::from([BQNValue::from(0.0), BQNValue::from(err)]),
    };
    ret.copy()
}

fn bqn_repl_init(ctx: &mut Context) -> BQNValue {
    BQNValue::init();

    let replfun = BQNValue::eval(include_str!("./repl.bqn"));

    let obj = ctx.as_bqnvalue();
    let replarg = BQNValue::from([
        BQNValue::fn2(bqn_notebook_input, &obj),
        BQNValue::fn2(bqn_notebook_output, &obj),
        BQNValue::fn2(bqn_notebook_display, &obj),
        BQNValue::fn1(bqn_notebook_clear, &obj),
        BQNValue::fn1(bqn_notebook_png, &obj),
        BQNValue::fn2(bqn_notebook_wav, &obj),
    ]);

    BQNValue::call1(&replfun, &replarg)
}

fn bqn_repl_exec(repl: &BQNV, post: &BQNV, silent: bool, code: &str) -> Result<String, String> {
    let x = BQNValue::call_trap(repl, &BQNValue::from(code))?;
    let r = BQNValue::call_trap(post, &x)?;
    if silent {
        Ok("".to_owned())
    } else {
        Ok(r.to_string())
    }
}

fn bqn_repl_exec_with(
    repl: &BQNV,
    post: &BQNV,
    silent: bool,
    code: &str,
    with: &str,
) -> Result<String, String> {
    let with = BQNValue::call_trap(repl, &BQNValue::from(with))?;
    let post = if silent {
        BQNValue::from(post)
    } else {
        let p = BQNValue::call_trap(&with, &BQNValue::from("•fmt"));
        if let Ok(p) = p {
            p
        } else {
            BQNValue::from(post)
        }
    };
    bqn_repl_exec(&with, &post, silent, code)
}

fn bqn_completion(comp: &BQNV, code: &str, pos: f64) -> (Vec<String>, i64, i64) {
    let code = BQNValue::from(code);
    let pos = BQNValue::from(pos);
    let res = BQNValue::call2(comp, &code, &pos);
    let matches = res.pick(0).to_string_vec();
    let cursor_start = res.pick(1).to_f64() as i64;
    let cursor_end = res.pick(2).to_f64() as i64;

    (matches, cursor_start, cursor_end)
}

#[derive(PartialEq)]
enum BQNBrackets {
    Paren,  // ()
    Square, // []
    Curly,  // {}
    Vector, // ⟨⟩
}

macro_rules! left_bracket {
    ($bracket:ident, $stack:ident) => {
        $stack.push(BQNBrackets::$bracket)
    };
}
macro_rules! right_bracket {
    ($bracket:ident, $stack:ident) => {{
        let top = $stack.pop();
        if let Some(top) = top {
            if top != BQNBrackets::$bracket {
                return "invalid";
            }
        } else {
            return "invalid";
        }
    }};
}
fn bqn_is_complete(code: &str) -> &str {
    if code.starts_with(")off") {
        return "complete";
    }

    let mut code = code;
    while code.starts_with(')') {
        (_, code) = code.split_once('\n').unwrap_or(("", ""));
        if code.is_empty() {
            return "incomplete";
        }
    }

    let mut n_char = 0i8;
    let mut in_str = false;
    let mut stack = Vec::new();
    for c in code.chars() {
        if n_char == 1 {
            n_char = 2;
            continue;
        }
        if in_str && c != '"' {
            continue;
        }
        match c {
            '(' => left_bracket!(Paren, stack),
            ')' => right_bracket!(Paren, stack),
            '[' => left_bracket!(Square, stack),
            ']' => right_bracket!(Square, stack),
            '{' => left_bracket!(Curly, stack),
            '}' => right_bracket!(Curly, stack),
            '⟨' => left_bracket!(Vector, stack),
            '⟩' => right_bracket!(Vector, stack),
            '\'' => {
                if n_char == 0 {
                    n_char = 1;
                } else if n_char == 2 {
                    n_char = 0;
                }
            }
            '"' => {
                in_str = !in_str;
            }
            _ => {}
        }
    }
    if in_str {
        "incomplete"
    } else if stack.is_empty() {
        "complete"
    } else {
        "incomplete"
    }
}

fn shell_execute(key: &str, shell: Socket, iopub: Socket, stdin: Socket) {
    let mut ctx = Context {
        key: key.to_owned(),
        last_msg: Message::default(),
        iopub,
        stdin,
    };

    let mut execution_count: i32 = 0;

    let repl = bqn_repl_init(&mut ctx);

    let comp = {
        let c = BQNValue::call_trap(&repl, &BQNValue::from(include_str!("./completion.bqn")))
            .expect("Completion should work");
        BQNValue::call_trap(&c, &repl).expect("Completion should work")
    };

    let fmt = BQNValue::eval("•fmt");
    let nil = BQNValue::eval("{S: @}");
    let img =
        BQNValue::call_trap(&repl, &BQNValue::from("•jupyter.image")).expect("Png should work");
    let aud =
        BQNValue::call_trap(&repl, &BQNValue::from("•jupyter.audio")).expect("Wav should work");

    loop {
        let msg = recv_msg(&shell, key);

        ctx.update_msg(msg.clone());

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
        ctx.send_iopub(busy);

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
                ctx.send_iopub(input);

                // this field is deprecated according to the docs
                let mut payload: Vec<Value> = Vec::new();

                let mut silent = msg.content["silent"].as_bool().unwrap_or(false);
                let mut post = None;
                let mut with = None;
                let rslt = loop {
                    if code.starts_with(')') {
                        let (cmd, next) = code.split_once('\n').unwrap_or((code, ""));
                        code = next;

                        if cmd.starts_with(")r") {
                            silent = true;
                            post = Some(&nil);
                        } else if cmd.starts_with(")use") {
                            let (_, r) = cmd.split_once(' ').unwrap_or(("", ""));
                            with = Some(r);
                        } else if cmd.starts_with(")off") {
                            payload.push(json!({
                                "source": "ask_exit",
                                "keepkernel": false,
                            }));
                        } else if cmd.starts_with(")image") {
                            silent = true;
                            post = Some(&img);
                        } else if cmd.starts_with(")audio") {
                            silent = true;
                            post = Some(&aud);
                        } else {
                            break Err(format!("Unknown command {cmd}"));
                        }
                    } else if code.trim().is_empty() {
                        silent = true;
                        break Ok("".to_owned());
                    } else {
                        let post = post.unwrap_or(&fmt);
                        if let Some(with) = with {
                            break bqn_repl_exec_with(&repl, post, silent, code, with);
                        } else {
                            break bqn_repl_exec(&repl, post, silent, code);
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
                        ctx.send_iopub(ex_rs);
                    }

                    reply_msg(
                        &msg,
                        json!({
                            "status": "ok",
                            "execution_count": execution_count,
                            "payload": payload,
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
                    ctx.send_iopub(ex_rs);

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
            Some("complete_request") => {
                let code = msg.content["code"].as_str().unwrap();
                let pos = msg.content["cursor_pos"].as_f64().unwrap();

                let (matches, cursor_start, cursor_end) = bqn_completion(&comp, code, pos);

                let re = reply_msg(
                    &msg,
                    json!({
                        "status": "ok",
                        "matches": matches,
                        "cursor_start": cursor_start,
                        "cursor_end": cursor_end,
                        "metadata": {},
                    }),
                );
                send_msg(&shell, key, re);
            }
            Some("is_complete_request") => {
                let code = msg.content["code"].as_str().unwrap();
                let re = reply_msg(
                    &msg,
                    json!({
                        "status": bqn_is_complete(code),
                    }),
                );
                send_msg(&shell, key, re);
            }
            Some(_) | None => {}
        }
        ctx.send_iopub(idle);
    }
}

fn run(filename: String) {
    let ci = read_connection_file(filename);

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

    thread::spawn(move || loop {
        let heartbeat = hb.recv_bytes(0).unwrap();
        hb.send(&heartbeat, 0).unwrap();
    });

    let ci_key = ci.key.clone();
    thread::spawn(move || {
        shell_execute(&ci_key, shell, iopub, stdin);
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
                run(filename)
            }
            _ => panic!("Unknown option"),
        }
    } else {
        create_kernel_json();
    }
}
