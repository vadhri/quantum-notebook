use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use config::Config;
use std::collections::HashMap;
use std::{thread, time};

#[macro_use]
extern crate lazy_static;

use aes::Aes128;
use aes::cipher::{
    BlockEncrypt,
    generic_array::GenericArray,
};
use aes::NewBlockCipher;
use std::sync::Mutex;

lazy_static! {
    static ref KEYLOOKUP: Mutex<HashMap<u32, String>> = Mutex::new(HashMap::new());
}

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
                // println!("size {:?} {:?}", size, &data[0..size]);
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
        println!("key rotation {:?}", map.get(&KEY_ID).unwrap());
    }
}

fn main() {
    let mut map = KEYLOOKUP.lock().unwrap();
    map.insert(0, "0000000000000000".to_string());
    std::mem::drop(map);

    let cfg = Config::builder()
        .add_source(config::File::with_name("config.json"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>().unwrap();

        let connection_string = format!("{}:{}", cfg["bind_ip"], cfg["key_recv_port_sender"]);
        println!("connecting @ {:#?}", connection_string.clone());
        
        let thandle = thread::spawn(|| {
            let listener = TcpListener::bind(connection_string).unwrap();
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        thread::spawn(move|| {
                            handle_incoming_key(stream)
                        });
                    }
                    Err(_e) => {
                        println!("Failed to connect: {}", _e);
                    }
                }
            }
            drop(listener);
        });
        
    let connection_string = format!("{}:{}", cfg["bind_ip"], cfg["data_transmission_port"]);

    thread::spawn(move || {    
        match TcpStream::connect(connection_string.clone()) {
            Ok(mut stream) => {
                println!("Successfully connected to server in port {:?}", connection_string);
                let filename: &str = "test.txt";
                let bytes = std::fs::read(filename).unwrap();

                for mut bytes in bytes.chunks_exact(16) {
                    let ten_millis = time::Duration::from_millis(250);                    
                    thread::sleep(ten_millis);                    
                    let mut dp = Vec::new();
                    
                    unsafe {
                        let map = KEYLOOKUP.lock().unwrap();
                        let chunk_key = map.get(&KEY_ID).unwrap();
                        
                        let generic_array_key = GenericArray::clone_from_slice(chunk_key.as_bytes());
                        let mut generic_array_data = GenericArray::clone_from_slice(&mut bytes);
                        
                        let cipher = Aes128::new(&generic_array_key);
                        cipher.encrypt_block(&mut generic_array_data);
                        dp.append(&mut vec![KEY_ID as u8]);
                        dp.append(&mut generic_array_data.to_vec().clone());
                        stream.write(&dp.to_vec()).unwrap();
                        
                        std::mem::drop(map);
                    }
                    
                }
            },
            Err(_e) => {
                println!("Failed to connect: {}", _e);
            }
        }
    });

    thandle.join().unwrap();

    println!("Terminated.");
}