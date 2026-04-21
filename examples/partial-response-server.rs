use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use modbus::{
    adu::tcp::response::Response as AduResponse,
    pdu::{DataWords, response::Response as PduResponse},
};

fn main() {
    let socket_addr = "localhost:5502";
    let listener = TcpListener::bind(socket_addr).unwrap();
    println!("Modbus server listening on {socket_addr}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_connection(stream));
            }
            Err(e) => {
                eprintln!("Failed creating a connection with error: {e}")
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("New connection created");

    let mut buf = [0; 11]; // [0, 1, 0, 0, 0, 5, 1, 4, 2, 1, 2]
    AduResponse::new(
        1,
        1,
        Ok(PduResponse::ReadInputRegisters(DataWords::new(&[1, 2], 2))),
    )
    .encode(&mut buf)
    .unwrap();

    for b in buf.chunks(1) {
        thread::sleep(Duration::from_millis(100));
        println!("Writing {b:?}");
        stream.write_all(b).unwrap();
    }
}
