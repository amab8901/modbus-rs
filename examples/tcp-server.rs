use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use modbus::{
    adu::tcp::{header::Header, request::Request as AduRequest, response::Response as AduResponse},
    error::DecodeError,
    exception_code::ExceptionCode,
    pdu::{
        DataWords, exception_response::ExceptionResponse as PduExceptionResponse,
        function_code::FunctionCode, request::Request as PduRequest,
        response::Response as PduResponse,
    },
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

    let mut req_buf: Vec<u8> = vec![];
    loop {
        let mut tmp_req_buf = [0; 300];
        let bytes_read = match stream.read(&mut tmp_req_buf) {
            Ok(bytes_read) => bytes_read,
            Err(err) => {
                eprintln!("Failed reading stream with error: {err}");
                return;
            }
        };
        println!("{bytes_read} bytes were received");

        if bytes_read == 0 {
            println!("EOF");
            return;
        }
        req_buf.extend_from_slice(&tmp_req_buf[..bytes_read]);
        println!("req_buf: {req_buf:?}");

        if req_buf.len() < Header::size() {
            let current_size = req_buf.len();
            let min_needed_size = Header::size();
            println!("Incomplete buffer: {current_size}/{min_needed_size}");
            continue;
        };
        let (req_header_buf, req_pdu_buf) = req_buf.split_at(Header::size());

        let header = Header::try_from(req_header_buf).unwrap();
        if *header.unit_id() == 111 {
            // Disallow unit_id 111. Hopefully no one got screwed by disallowing 111 xD
            // Can be changed to only allow unit_id 1 or something (header.unit_id != 1).
            // This is more for showing where to put the check.
            return;
        }

        if *header.length() as usize > req_pdu_buf.len() + 1 {
            let current_size = req_buf.len();
            let min_needed_size = *header.length() as usize + Header::size() - 1;
            println!("Incomplete buffer: {current_size}/{min_needed_size}");
            continue;
        };

        let pdu_req = match PduRequest::try_from(req_pdu_buf) {
            Ok(req) => req,
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
        };

        let req = AduRequest::new(*header.transaction_id(), *header.unit_id(), pdu_req);
        println!("{req:?}");

        let pdu_res = match req.pdu() {
            PduRequest::ReadInputRegisters(_, _) => Ok(PduResponse::ReadInputRegisters(
                DataWords::new(&[0x01, 0x02], 1),
            )),
            _ => Err(PduExceptionResponse::new(
                FunctionCode::from(req.pdu()),
                ExceptionCode::IllegalFunction,
            )),
        };

        let res = AduResponse::new(
            *req.header().transaction_id(),
            *req.header().unit_id(),
            pdu_res,
        );
        println!("{res:?}");
        let mut res_buf = vec![0; res.adu_len()];
        let size = res.encode(&mut res_buf).unwrap();
        println!("res_buf: {res_buf:?}");
        println!("{size:?}");

        let _ = stream.write_all(&res_buf);
    }
}
