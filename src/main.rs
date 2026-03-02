use std::{io::{BufRead, BufReader}, net::{TcpListener, TcpStream}};

fn handle_connection(stream: TcpStream){
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    match reader.read_line(&mut buffer) {
        Ok(_) => {},
        Err(err) => {println!("{}",err)},
    };

    println!("{}",buffer)
}

fn main(){
    let listener = TcpListener::bind("localhost:9876").unwrap();

    for stream in listener.incoming(){
        match stream {
            Ok(stream) => {handle_connection(stream);},
            Err(error) => println!("{}",error),
        }
    }
}
