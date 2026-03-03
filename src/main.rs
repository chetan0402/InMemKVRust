use std::{
    cell::RefCell,
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

thread_local! {
    static MAP: RefCell<HashMap<String,String>> = RefCell::new(HashMap::new());
}

fn handle_connection(stream: TcpStream) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    reader
        .read_line(&mut buffer)
        .map_err(|err| format!("read err: {}", err))?;

    print!("{}", buffer);
    let mut iter = buffer.split_whitespace();
    match iter.next() {
        Some("SET") => {
            let key = iter.next().ok_or("invalid command")?.to_string();
            let value = iter.next().ok_or("invalid command")?.to_string();
            MAP.with(|m| {
                m.borrow_mut().insert(key, value);
            });
        }
        Some("GET") => {
            let value = {
                let key = iter.next().ok_or("invalid command")?.to_string();
                MAP.with(|m| m.borrow().get(&key).cloned())
            };

            reader
                .get_mut()
                .write(value.ok_or("key not found")?.as_bytes())
                .map_err(|err| format!("write err:{}", err))?;
        }
        Some(command) => return Err(format!("unknown command: {}", command)),
        None => return Err("No command found".to_string()),
    }

    Ok(())
}

fn main() {
    let listener = TcpListener::bind("localhost:9876").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream).unwrap_or_else(|e| eprintln!("{}",e))
            }
            Err(error) => println!("{}", error),
        }
    }
}
