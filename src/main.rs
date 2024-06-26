mod media;
mod messaging;
use cbqn_sys::{
    bqn_bound, bqn_call1, bqn_call2, bqn_copy, bqn_eval, bqn_free, bqn_init, bqn_makeBoundFn1,
    bqn_makeBoundFn2, bqn_makeChar, bqn_makeF64, bqn_makeObjVec, bqn_makeUTF8Str, bqn_pick,
    bqn_readC8Arr, bqn_readF64, bqn_readF64Arr, bqn_shape, BQNV,
};
use media::{base64_png, base64_wav};
use messaging::{
    format_address, new_msg, read_connection_file, recv_msg, reply_msg, send_msg, Message,
};
use serde_json::{json, Value};
use std::{
    ffi::c_char,
    fs::File,
    sync::{Mutex, Once},
    thread,
};
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

static mut TO_UTF8: BQNV = 0;
static INIT_TO_UTF8: Once = Once::new();

// consumes a
unsafe fn str_from_bqnv(a: BQNV) -> String {
    INIT_TO_UTF8.call_once(|| {
        TO_UTF8 = bqn_eval_str("•ToUTF8");
    });

    let utf8 = bqn_call1(TO_UTF8, a);
    let bound = bqn_bound(utf8);
    let mut bytes = Vec::with_capacity(bound);
    bytes.set_len(bound);
    bqn_readC8Arr(utf8, bytes.as_mut_ptr());
    bqn_free(a);
    bqn_free(utf8);

    String::from_utf8(bytes).expect("Should be valid UTF8")
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

unsafe extern "C" fn bqn_notebook_png(obj: BQNV, x: BQNV) -> BQNV {
    bqn_free(obj);
    // rank has to be = 3
    let mut shape = [0usize; 3];
    bqn_shape(x, shape.as_mut_ptr());
    let height = shape[0] as u32;
    let width = shape[1] as u32;
    let channels = shape[2] as u32;

    let bound = bqn_bound(x);
    let mut img = Vec::with_capacity(bound);
    img.set_len(bound);
    bqn_readF64Arr(x, img.as_mut_ptr());
    let data: Vec<u8> = img.iter().map(|&f| (f * 255.0) as u8).collect();
    let png = base64_png(width, height, channels, &data);
    let ret = match png {
        Ok(png) => bqn_makeObjVec(2, [bqn_makeF64(1.0), str_to_bqnv(&png)].as_ptr()),
        Err(err) => bqn_makeObjVec(2, [bqn_makeF64(0.0), str_to_bqnv(&err)].as_ptr()),
    };
    bqn_free(x);
    ret
}

unsafe extern "C" fn bqn_notebook_wav(obj: BQNV, w: BQNV, x: BQNV) -> BQNV {
    bqn_free(obj);
    // rank has to be = 2
    let mut shape = [0usize; 2];
    bqn_shape(x, shape.as_mut_ptr());
    let channels = shape[1] as u16;

    let sample_rate = f64_from_bqnv(w) as u32;

    let bound = bqn_bound(x);
    let mut aud = Vec::with_capacity(bound);
    aud.set_len(bound);
    bqn_readF64Arr(x, aud.as_mut_ptr());
    let wav = base64_wav(channels, sample_rate, &aud);
    let ret = match wav {
        Ok(wav) => bqn_makeObjVec(2, [bqn_makeF64(1.0), str_to_bqnv(&wav)].as_ptr()),
        Err(err) => bqn_makeObjVec(2, [bqn_makeF64(0.0), str_to_bqnv(&err)].as_ptr()),
    };
    bqn_free(x);
    ret
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

    let arg = [
        bqn_makeBoundFn2(Some(bqn_notebook_input), obj),
        bqn_makeBoundFn2(Some(bqn_notebook_output), obj),
        bqn_makeBoundFn2(Some(bqn_notebook_display), obj),
        bqn_makeBoundFn1(Some(bqn_notebook_clear), obj),
        bqn_makeBoundFn1(Some(bqn_notebook_png), obj),
        bqn_makeBoundFn2(Some(bqn_notebook_wav), obj),
    ];
    let replarg = bqn_makeObjVec(arg.len(), arg.as_ptr());

    let repl = bqn_call1(replfun, replarg);

    bqn_free(replarg);
    bqn_free(replfun);
    bqn_free(obj);

    repl
}

static mut TRAP: BQNV = 0;
static INIT_TRAP: Once = Once::new();

// consumes all
unsafe fn bqn_repl_eval(eval: BQNV, x: BQNV) -> Result<BQNV, String> {
    INIT_TRAP.call_once(|| {
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
    bqn_free(eval);
    bqn_free(x);
    bqn_free(r);
    ret
}

unsafe fn bqn_repl_exec(
    repl: BQNV,
    post: BQNV,
    silent: bool,
    code: &str,
) -> Result<String, String> {
    let x = bqn_repl_eval(repl, str_to_bqnv(code))?;
    let r = bqn_repl_eval(post, x)?;
    if silent {
        bqn_free(r);
        Ok("".to_owned())
    } else {
        Ok(str_from_bqnv(r))
    }
}

unsafe fn bqn_repl_exec_with(
    repl: BQNV,
    post: BQNV,
    silent: bool,
    code: &str,
    with: &str,
) -> Result<String, String> {
    let with = bqn_repl_eval(repl, str_to_bqnv(with))?;
    let post = if silent {
        post
    } else {
        let p = bqn_repl_eval(bqn_copy(with), str_to_bqnv("•fmt"));
        if let Ok(p) = p {
            bqn_free(post);
            p
        } else {
            post
        }
    };
    bqn_repl_exec(with, post, silent, code)
}

unsafe fn bqn_completion(comp: BQNV, code: &str, pos: f64) -> (Vec<String>, i64, i64) {
    let code = str_to_bqnv(code);
    let pos = bqn_makeF64(pos);
    let res = bqn_call2(comp, code, pos);
    let vec = bqn_pick(res, 0);
    let cursor_start = f64_from_bqnv(bqn_pick(res, 1)) as i64;
    let cursor_end = f64_from_bqnv(bqn_pick(res, 2)) as i64;

    let bound = bqn_bound(vec);
    let mut matches = Vec::with_capacity(bound);
    for i in 0..bound {
        let s = str_from_bqnv(bqn_pick(vec, i));
        matches.push(s);
    }
    bqn_free(vec);

    bqn_free(code);
    bqn_free(pos);
    bqn_free(res);

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
    while code.starts_with(")") {
        (_, code) = code.split_once("\n").unwrap_or(("", ""));
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

fn shell_execute(key: &str, shell: Socket) {
    let mut execution_count: i32 = 0;

    let repl = unsafe { bqn_repl_init() };

    let comp = unsafe {
        bqn_repl_eval(
            bqn_copy(repl),
            str_to_bqnv(include_str!("./completion.bqn")),
        )
        .expect("Completion should work")
    };

    let fmt = unsafe { bqn_eval_str("•fmt") };
    let img = unsafe {
        bqn_repl_eval(bqn_copy(repl), str_to_bqnv("•jupyter.image")).expect("Png should work")
    };
    let aud = unsafe {
        bqn_repl_eval(bqn_copy(repl), str_to_bqnv("•jupyter.audio")).expect("Wav should work")
    };

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

                // this field is deprecated according to the docs
                let mut payload: Vec<Value> = Vec::new();

                let mut silent = msg.content["silent"].as_bool().unwrap_or(false);
                let mut post = None;
                let mut with = None;
                let rslt = loop {
                    if code.starts_with(")") {
                        let (cmd, next) = code.split_once("\n").unwrap_or((code, ""));
                        code = next;

                        if cmd.starts_with(")r") {
                            silent = true;
                        } else if cmd.starts_with(")use") {
                            let (_, r) = cmd.split_once(" ").unwrap_or(("", ""));
                            with = Some(r);
                        } else if cmd.starts_with(")off") {
                            payload.push(json!({
                                "source": "ask_exit",
                                "keepkernel": false,
                            }));
                        } else if cmd.starts_with(")image") {
                            silent = true;
                            post = Some(unsafe { bqn_copy(img) });
                        } else if cmd.starts_with(")audio") {
                            silent = true;
                            post = Some(unsafe { bqn_copy(aud) });
                        } else {
                            break Err(format!("Unknown command {cmd}"));
                        }
                    } else if code.trim().is_empty() {
                        silent = true;
                        break Ok("".to_owned());
                    } else {
                        let post = post.unwrap_or(unsafe { bqn_copy(fmt) });
                        if let Some(with) = with {
                            break unsafe {
                                bqn_repl_exec_with(bqn_copy(repl), post, silent, code, with)
                            };
                        } else {
                            break unsafe { bqn_repl_exec(bqn_copy(repl), post, silent, code) };
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
            Some("complete_request") => {
                let code = msg.content["code"].as_str().unwrap();
                let pos = msg.content["cursor_pos"].as_f64().unwrap();

                let (matches, cursor_start, cursor_end) =
                    unsafe { bqn_completion(comp, code, pos) };

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
        send_iopub(key, idle);
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
                return run(filename);
            }
            _ => panic!("Unknown option"),
        }
    } else {
        create_kernel_json();
    }
}
