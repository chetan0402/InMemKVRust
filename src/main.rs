use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

thread_local! {
    static MAP: RefCell<HashMap<String,String>> = RefCell::new(HashMap::new());
}

fn process_command(command: &String) -> Result<String, Box<dyn Error>> {
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

            return value.ok_or("key not found".into());
        }
        Some(command) => return Err(format!("unknown command: {}", command).into()),
        None => return Err("No command found".into()),
    }

    Ok(String::new())
}

fn handle_connection(stream: TcpStream, wal: &mut File) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    reader.read_line(&mut buffer)?;

    wal.write_all(buffer.as_bytes())?;

    reader
        .get_mut()
        .write(process_command(&buffer)?.as_bytes())?;

    Ok(())
}

fn restore_wal(wal: &mut File) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::new(wal);

    for command in reader.lines().map(|c| c.unwrap()) {
        process_command(&command)?;
    }

    Ok(())
}

fn main() {
    let mut wal = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("WAL.log")
        .unwrap();
    restore_wal(&mut wal).unwrap();
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
