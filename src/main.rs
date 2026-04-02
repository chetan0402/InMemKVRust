use core::fmt;
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use epoll_rs::{Epoll, EpollEvent, Opts};

thread_local! {
    static MAP: RefCell<HashMap<String,String>> = RefCell::new(HashMap::new());
}

const SET_ERR_MSG_SYNTAX: &str = "err: SET <key> <value>";
const GET_ERR_MSG_SYNTAX: &str = "err: GET <key>";
const DELETE_ERR_MSG_SYNTAX: &str = "err: DELETE <key>";

#[derive(Debug, Clone)]
struct CloseFD;

impl Error for CloseFD {}

impl fmt::Display for CloseFD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Close FD")
    }
}

fn process_command(command: &String) -> Result<String, Box<dyn Error>> {
    let mut iter = command.split_whitespace();
    match iter.next() {
        Some("SET") => {
            let key = iter.next().ok_or(SET_ERR_MSG_SYNTAX)?.to_string();
            let value = iter.next().ok_or(SET_ERR_MSG_SYNTAX)?.to_string();
            MAP.with_borrow_mut(|m| m.insert(key, value));
        }
        Some("GET") => {
            let value = {
                let key = iter.next().ok_or(GET_ERR_MSG_SYNTAX)?.to_string();
                MAP.with_borrow(|m| m.get(&key).cloned())
            };

            return value.ok_or("err: key not found".into());
        }
        Some("DELETE") => {
            let key = iter.next().ok_or(DELETE_ERR_MSG_SYNTAX)?.to_string();
            MAP.with_borrow_mut(|m| m.remove(&key));
        }
        Some(command) => return Err(format!("err: Unknown command: {}", command).into()),
        None => return Err(CloseFD.into()),
    }

    Ok(String::new())
}

fn handle_connection(stream: &TcpStream, wal: &mut File) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    reader.read_line(&mut buffer)?;

    dbg!(&buffer);

    wal.write_all(buffer.as_bytes())?;

    let resp = match process_command(&buffer) {
        Ok(val) => val,
        Err(e) => {
            if let Some(_) = e.downcast_ref::<CloseFD>() {
                return Err(CloseFD.into());
            }
            e.to_string()
        }
    };

    dbg!(&resp);

    reader.get_mut().write(resp.as_bytes())?;

    Ok(())
}

fn restore_wal(wal: &mut File) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::new(wal);

    for command in reader.lines().map(|c| c.unwrap()) {
        let _ = process_command(&command);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut wal = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("WAL.log")?;
    restore_wal(&mut wal)?;
    let listener = TcpListener::bind("localhost:9876")?;
    listener.set_nonblocking(true)?;

    let epoll = Epoll::new()?;
    let tcp_sock = epoll.add(listener, Opts::IN)?;

    let mut events = [EpollEvent::zeroed(); 32];
    let mut clients = HashMap::<i32, TcpStream>::new();

    loop {
        let n = epoll.wait(&mut events)?.len();

        for i in 0..n {
            let event = events[i];

            if event.fd() == tcp_sock.fd() {
                while let Ok((stream, _)) = tcp_sock.file().accept() {
                    let res = || -> Result<(), Box<dyn Error>> {
                        stream.set_nonblocking(true)?;
                        let file = epoll.add(stream, Opts::IN)?;
                        clients.insert(file.fd(), file.into_file());
                        Ok(())
                    }();
                    if let Err(e) = res {
                        eprintln!("err: {}", e);
                    }
                }
            } else {
                if let Some(stream) = clients.remove(&event.fd()) {
                    if let Err(e) = handle_connection(&stream, &mut wal) {
                        eprintln!("err: {}", e);
                    } else {
                        clients.insert(event.fd(), stream);
                    }
                }
            }
        }
    }
}
