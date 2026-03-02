use std::{cell::RefCell, collections::HashMap, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};

thread_local! {
    static MAP: RefCell<HashMap<String,String>> = RefCell::new(HashMap::new());
}

fn handle_connection(stream: TcpStream){
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    match reader.read_line(&mut buffer) {
        Ok(_) => {},
        Err(err) => {println!("{}",err)},
    };

    buffer=buffer.trim_end().to_string();
    println!("{}",buffer);
    let tokens: Vec<&str> = buffer.split(" ").collect();
    if tokens[0]=="SET"{
        MAP.with(|m|{
            m.borrow_mut().insert(tokens[1].to_string(), tokens[2].to_string());
            println!("Insert@{}:{}",tokens[1].to_string(),tokens[2].to_string());
        });
    }
    if tokens[0]=="GET"{
        let mut value: String = String::new();
        MAP.with(|m|{
            match m.borrow().get(&tokens[1].to_string()){
                None => {println!("Key({}) not present",tokens[1].to_string())},
                Some(v) => {value=v.clone()}
            };

        });
        if reader.get_mut().write(value.as_bytes()).is_err(){
            println!("Response failed");
        }
    }
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
