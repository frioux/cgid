use std::env;
use std::io;
use std::io::prelude::*;
use std::process::Command;
use std::process::Stdio;

enum Header {
    Key,
    WS,
    Value
}

fn set_header(line: String) {
    let mut key: Vec<char> = Vec::new();
    let mut value: Vec<char> = Vec::new();
    let mut state = Header::Key;
    for c in line.chars() {
        match state {
            Header::Key => {
                if c == ':' {
                    state = Header::WS;
                } else {
                    key.push(c)
                }
            }
            Header::WS => {
                // maybe should skip tabs too?
                if c != ' ' {
                    value.push(c);
                    state = Header::Value;
                }
            }
            Header::Value => {
                value.push(c)
            }
        }
    }
    let mut env_key = "HTTP_".to_owned();
    env_key.push_str(&key.iter().cloned().collect::<String>()
        .to_uppercase()
        .replace("-", "_")
    );
    let env_value = value.iter().cloned().collect::<String>();
    env::set_var(env_key, env_value);
}

enum Req {
    Method,
    ScriptName,
    QueryString,
    Protocol
}

// https://www.ietf.org/rfc/rfc3875
fn set_request(line: String) {
    let mut method: Vec<char> = Vec::new();
    let mut script_name: Vec<char> = Vec::new();
    let mut query_string: Vec<char> = Vec::new();
    let mut server_protocol: Vec<char> = Vec::new();
    let mut state = Req::Method;

    for c in line.chars() {
        match state {
            Req::Method => {
                if c == ' ' {
                    state = Req::ScriptName;
                } else {
                    method.push(c);
                }
            }
            Req::ScriptName => {
                if c == '?' {
                    state = Req::QueryString;
                } else if c == ' ' {
                    state = Req::Protocol;
                } else {
                    script_name.push(c);
                }
            }
            Req::QueryString => {
                if c == ' ' {
                    state = Req::Protocol;
                } else {
                    query_string.push(c);
                }
            }
            Req::Protocol => {
                server_protocol.push(c);
            }
        }
    }

    env::set_var("REQUEST_METHOD", method.iter().cloned().collect::<String>());
    env::set_var("SCRIPT_NAME", script_name.iter().cloned().collect::<String>());
    env::set_var("PATH_INFO", script_name.iter().cloned().collect::<String>());
    env::set_var("QUERY_STRING", query_string.iter().cloned().collect::<String>());
    env::set_var("REQUEST_URI", "/cgi-bin/app.cgi");
    env::set_var("SERVER_PROTOCOL", server_protocol.iter().cloned().collect::<String>());
}

fn main() {
    env::set_var("GATEWAY_INTERFACE", "CGI/1.1");
    env::set_var("SERVER_SOFTWARE", "httpd.rs/0.0.1");
    // Maybe give a better error if this is unset
    env::set_var("SERVER_NAME", env::var("TCPLOCALIP").unwrap());
    env::set_var("SERVER_PORT", env::var("TCPLOCALPORT").unwrap());

    let stdin = io::stdin();

    let mut req = String::new();
    stdin.lock().read_line(&mut req);

    set_request(req);

    // The following could likely be done better with a regex
    for line in stdin.lock().lines() {
        let val = line.unwrap();
        if val == "" {
            break;
        }
        set_header(val)
    }

    let args: Vec<_> = env::args().collect();

    let mut child: Command = Command::new(args[1].clone());
    for i in 2..args.len() {
        child.arg(args[i].clone());
    }
    child.stdin(Stdio::piped())
        .stdout(Stdio::piped());
    // Handle possible errors here?
    let f = child.spawn().unwrap();

    // Handle possible errors here?
    io::copy(&mut io::stdin(), &mut f.stdin.unwrap());
    io::copy(&mut f.stdout.unwrap(), &mut io::stdout());
}
