use clap::Parser;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::net::SocketAddrV4;
use std::net::TcpStream;

pub static scripts_repository: &str = "bash_scripts";

#[derive(Parser, Debug)]
pub struct DispatcherConfig {
    #[clap(long, default_value = "127.0.0.1:9000")]
    pub socket: SocketAddrV4,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Status,
    Update(String),
    Register(SocketAddrV4),
    Dispatch(String),
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Ok,
    Error(String),
}

pub fn send_massage(socket: &SocketAddrV4, request: Request) -> Response {
    match TcpStream::connect(socket) {
        Ok(mut stream) => {
            info!("Sending request: {:?}", request);
            serde_json::to_writer(&stream, &request).unwrap();
            stream.flush().unwrap();

            let mut deserializer =
                serde_json::Deserializer::from_reader(stream.try_clone().unwrap());

            match Response::deserialize(&mut deserializer) {
                Ok(_) => Response::Ok,
                Err(err) => {
                    error!("Error while reading response: {:?}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            error!("Error: {:?} while trying to connect to socket", error);
            std::process::exit(1);
        }
    }
}

pub fn check_status(socket: &SocketAddrV4) -> Response {
    info!("Checking status of dispatcher");
    send_massage(socket, Request::Status)
}
