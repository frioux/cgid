use std::env;
use std::io;
use std::io::prelude::*;
use std::process::Command;
use std::process::Stdio;

enum HTTP {
    _400,
    _500,
}

enum Header {
    Key,
    WS,
    Value
}

macro_rules! warn {
    ($fmt:expr) => (writeln!(io::stderr(), $fmt));
    ($fmt:expr, $($arg:tt)*) => (writeln!(io::stderr(), $fmt, $($arg)*));
}

fn early_exit(line: &str) -> ! {
    print!("HTTP/1.0 {}\r\n", line);
    std::process::exit(1);
}

fn set_header(line: String, content_length: &mut usize) -> Result<(), HTTP> {
    let mut key: Vec<char> = Vec::new();
    let mut value: Vec<char> = Vec::new();
    let mut state = Header::Key;
    let mut valid = false;
    for c in line.chars() {
        match state {
            Header::Key => {
                if c == ':' {
                    valid = true;
                    state = Header::WS;
                } else {
                    key.push(c)
                }
            }
            Header::WS => {
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
    if valid == false {
        return Err(HTTP::_400);
    };
    let mut env_key = "HTTP_".to_owned();
    env_key.push_str(&key.iter().cloned().collect::<String>()
        .to_uppercase()
        .replace("-", "_")
    );
    let env_value = value.iter().cloned().collect::<String>();

    if env_key == "HTTP_CONTENT_TYPE" {
        env_key = String::from("CONTENT_TYPE");
    } else if env_key == "HTTP_CONTENT_LENGTH" {
        env_key = String::from("CONTENT_LENGTH");
        match env_value.parse::<usize>() {
            Ok(n) => { *content_length = n },
            Err(_) => return Err(HTTP::_400),
        }
    }
    warn!("HEADER: {}={}", env_key, env_value);
    env::set_var(env_key, env_value);
    Ok(())
}

enum Req {
    Method,
    PathInfo,
    QueryString,
    Protocol
}

// https://www.ietf.org/rfc/rfc3875
fn set_request(line: String) {
    let mut method: Vec<char> = Vec::new();
    let mut path_info: Vec<char> = Vec::new();
    let mut query_string: Vec<char> = Vec::new();
    let mut server_protocol: Vec<char> = Vec::new();
    let mut state = Req::Method;

    for c in line.chars() {
        match state {
            Req::Method => {
                if c == ' ' {
                    warn!("METHOD: {}", method.iter().cloned().collect::<String>());
                    state = Req::PathInfo;
                } else {
                    method.push(c);
                }
            }
            Req::PathInfo => {
                if c == '?' {
                    state = Req::QueryString;
                    warn!("PATH_INFO: {}", path_info.iter().cloned().collect::<String>());
                } else if c == ' ' {
                    state = Req::Protocol;
                    warn!("PATH_INFO: {}", path_info.iter().cloned().collect::<String>());
                } else {
                    path_info.push(c);
                }
            }
            Req::QueryString => {
                if c == ' ' {
                    state = Req::Protocol;
                    warn!("QUERY_STRING: {}", query_string.iter().cloned().collect::<String>());
                } else {
                    query_string.push(c);
                }
            }
            Req::Protocol => {
                if c == '\n' {
                    warn!("SERVER_PROTOCOL: {}", server_protocol.iter().cloned().collect::<String>());
                    break;
                }
                server_protocol.push(c);
            }
        }
    }

    env::set_var("REQUEST_METHOD", method.iter().cloned().collect::<String>());
    env::set_var("SCRIPT_NAME", "");
    env::set_var("PATH_INFO", path_info.iter().cloned().collect::<String>());
    env::set_var("QUERY_STRING", query_string.iter().cloned().collect::<String>());
    env::set_var("REQUEST_URI", "/cgi-bin/app.cgi");
    env::set_var("SERVER_PROTOCOL", server_protocol.iter().cloned().collect::<String>());
}

fn main() {
    env::set_var("GATEWAY_INTERFACE", "CGI/1.1");
    env::set_var("SERVER_SOFTWARE", "httpd.rs/0.0.1");
    env::set_var("SERVER_NAME", env::var("TCPLOCALIP").unwrap_or_else(|e| {
        warn!("Couldn't get TCPLOCALIP (not running under UCSPI?)");
        warn!("Defaulting to 127.0.0.1");
        String::from("127.0.0.1")
    }));
    env::set_var("SERVER_PORT", env::var("TCPLOCALPORT").unwrap_or_else(|e| {
        warn!("Couldn't get TCPLOCALPORT (not running under UCSPI?)");
        warn!("Defaulting to 80");
        String::from("80")
    }));

    let stdin = io::stdin();

    let mut content_length: usize = 0;

    warn!("\n\n\n");
    let mut req = String::new();
    stdin.lock().read_line(&mut req);

    set_request(req);

    warn!("Request header set!\n");

    for line in stdin.lock().lines() {
        let val = line.unwrap_or_else(|e| {
            warn!("WTF how can there not be a line: {}", e);
            early_exit("500 Internal Server Error");
        });
        if val == "" {
            break;
        }
        match set_header(val, &mut content_length) {
            Ok(_) => (),
            Err(HTTP::_400) => early_exit("400 Invalid Header"),
            Err(e) => early_exit("500 Internal Server Error"),
        }
    }

    warn!("All headers set!\n");

    let args: Vec<_> = env::args().collect();

    let mut child: Command = Command::new(args[1].clone());
    for i in 2..args.len() {
        child.arg(args[i].clone());
    }
    child.stdin(Stdio::piped())
        .stdout(Stdio::piped());
    let f = child.spawn().unwrap_or_else(|e| {
        warn!("Failed to execute child: {}", e);
        early_exit("500 Internal Server Error");
    });

    // Handle possible errors here?

    let mut c_stdin = f.stdin.unwrap_or_else(|| {
        warn!("Failed to get child's STDIN");
        early_exit("500 Internal Server Error");
    });
    copy_exact(&mut io::stdin(), &mut c_stdin, content_length);
    warn!("Written.");

    // Note that this is where Content-Length would be recorded and passed, but
    // because it would incur more memory overhead and it would be a hassle, Content-Length is not
    // supported.  Maybe I'll add support optionally
    warn!("Writing STDIN to child's STDIN...");
    warn!("Writing child's STDOUT to STDOUT...");
    let mut c_stdout = f.stdout.unwrap_or_else(|| {
        warn!("Failed to get child's STDOUT");
        early_exit("500 Internal Server Error");
    });
    io::copy(&mut c_stdout, &mut io::stdout());
    warn!("Written.");
}

fn copy_exact<R: Read, W: Write>(mut reader: R, mut writer: W,
        length: usize) -> Result<(), std::io::Error> {
    const BUFFER_SIZE: usize = 64 * 1024;
    let mut buffer: Vec<u8> = vec![0; BUFFER_SIZE];

    let mut buffer_left = length;
    while buffer_left > BUFFER_SIZE {
        try!(reader.read_exact(&mut buffer));
        try!(writer.write_all(&buffer));
        buffer_left -= BUFFER_SIZE;
    }

    try!(reader.read_exact(&mut buffer[..buffer_left]));
    try!(writer.write_all(&buffer[..buffer_left]));
    Ok(())
}
