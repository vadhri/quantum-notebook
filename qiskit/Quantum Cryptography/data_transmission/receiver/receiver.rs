use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read};
use config::Config;
use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

lazy_static! {
    static ref KEYLOOKUP: Mutex<HashMap<u32, String>> = Mutex::new(HashMap::new());
}

use aes::Aes128;
use aes::cipher::{
    BlockDecrypt,
    generic_array::GenericArray,
};
use aes::NewBlockCipher;

static mut KEY_ID:u32 = 0;

fn handle_incoming_key(mut stream: TcpStream) {
    let mut data = [0 as u8; 32];
    let mut data_rcvd = Vec::new();

    while match stream.read(&mut data) {
        Ok(size) => {
            if size == 0 {
                false
            } else {
                let d = data.to_vec();
                data_rcvd.append(&mut d[0..size].to_vec());
                true
            }
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}

    let mut map = KEYLOOKUP.lock().unwrap();
    let key = std::str::from_utf8(data_rcvd.as_slice()).unwrap();
    
    unsafe {
        KEY_ID += 1;
        map.insert(KEY_ID, key.to_string().clone());
        println!("Key rotation {:?}", map.get(&KEY_ID).unwrap());
    }
    std::mem::drop(map);
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 33];
    let mut data_rcvd = Vec::new();
    
    while match stream.read(&mut data) {
        Ok(size) => {
            if size == 0 {
                false
            } else {
                let mut d = data.to_vec();
                let chunk_key_id = d[0];
                let map = KEYLOOKUP.lock().unwrap();
                let chunk_key = map.get(&(chunk_key_id as u32)).unwrap();
                                
                let generic_array_key = GenericArray::clone_from_slice(chunk_key.as_bytes());
                let mut generic_array_data = GenericArray::clone_from_slice(&mut d[1..size]);
                
                let cipher = Aes128::new(&generic_array_key);
                cipher.decrypt_block(&mut generic_array_data);

                data_rcvd.append(&mut generic_array_data.to_vec());
                println!("{:?} {:?} {:?}", chunk_key_id, size, " bytes received !");

                std::mem::drop(map);

                true
            }
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
    println!("Total_data {:?}", std::str::from_utf8(data_rcvd.as_slice()).unwrap());
}

fn main() {
    let cfg = Config::builder()
        .add_source(config::File::with_name("config.json"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>().unwrap();

    let connection_string = format!("{}:{}", cfg["bind_ip"], cfg["data_transmission_port"]);
    println!("connection string @ {:#?}", connection_string);
    
    let mut map = KEYLOOKUP.lock().unwrap();
    map.insert(0, "0000000000000000".to_string());
    std::mem::drop(map);

    let thandle = thread::spawn(|| {
        let listener = TcpListener::bind(connection_string).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Incoming data: {}", stream.peer_addr().unwrap());
                    thread::spawn(move|| {
                        // connection succeeded
                        handle_client(stream)
                    });
                }

                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
        drop(listener);
    });

    let connection_string = format!("{}:{}", cfg["bind_ip"], cfg["key_recv_port_recv"]);

    thread::spawn(|| {
        let listener = TcpListener::bind(connection_string).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New key received: {}", stream.peer_addr().unwrap());
                    thread::spawn(move|| {
                        // connection succeeded
                        handle_incoming_key(stream)
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
        drop(listener);
    });

    thandle.join().unwrap();
}
    