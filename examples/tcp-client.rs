use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use modbus::{
    adu::tcp::{request::Request as AduRequest, response::Response as AduResponse},
    error::DecodeError,
    pdu::request::Request as PduRequest,
};

fn main() {
    let addr: SocketAddr = "127.0.0.1:5502".parse().unwrap();
    let mut stream = TcpStream::connect(addr).unwrap();

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();

    let pdu_req = PduRequest::ReadInputRegisters(0, 1);
    let req = AduRequest::new(1, 1, pdu_req);
    println!("{req:?}");
    let mut req_buf = vec![0_u8; req.adu_len()];
    req.encode(&mut req_buf).unwrap();
    println!("req_buf: {req_buf:?}");
    stream.write_all(&req_buf).unwrap();
    stream.flush().unwrap();

    let mut tmp_res_buf: [u8; 300] = [0; 300];
    let mut buf_pos = 0;
    loop {
        let bytes_read = stream.read(&mut tmp_res_buf[buf_pos..]).unwrap();
        println!("{bytes_read} bytes were received");
        if bytes_read == 0 {
            println!("EOF");
            break;
        };
        buf_pos += bytes_read;
        let res_buf = &tmp_res_buf[..buf_pos];
        println!("res_buf: {res_buf:?}");

        match AduResponse::try_from(res_buf) {
            Ok(res) => {
                println!("{res:?}");
                break;
            }
            Err(err) => match err {
                DecodeError::IncompleteBuffer {
                    current_size,
                    min_needed_size,
                } => {
                    println!("Incomplete buffer: {current_size}/{min_needed_size}");
                    continue;
                }
                DecodeError::ModbusExceptionError(fn_code, exception_error) => {
                    println!("Modbus exception error: {fn_code:?} {exception_error:?}");
                    break;
                }
                DecodeError::ModbusExceptionCode(fn_code, exception_code) => {
                    println!("Modbus exception code: {fn_code:?} {exception_code:?}");
                    break;
                }
            },
        }
    }
}
