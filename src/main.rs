use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

thread_local! {
    static MAP: RefCell<HashMap<String,String>> = RefCell::new(HashMap::new());
}

fn process_command(command: &String) -> Result<String, String> {
    let mut iter = command.split_whitespace();
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

            return value.ok_or(String::from("key not found"));
        }
        Some(command) => return Err(format!("unknown command: {}", command)),
        None => return Err("No command found".to_string()),
    }

    Ok(String::from(""))
}

fn handle_connection(stream: TcpStream, wal: &mut File) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    reader
        .read_line(&mut buffer)
        .map_err(|err| format!("read err: {}", err))?;

    wal.write_all(buffer.as_bytes())
        .map_err(|err| format!("wal err: {}", err))?;

    reader
        .get_mut()
        .write(process_command(&buffer)?.as_bytes())
        .map_err(|err| err.to_string())?;

    Ok(())
}

fn main() {
    let mut wal = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("WAL.log")
        .unwrap();
    let listener = TcpListener::bind("localhost:9876").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream, &mut wal).unwrap_or_else(|e| eprintln!("{}", e))
            }
            Err(error) => println!("{}", error),
        }
    }
}
