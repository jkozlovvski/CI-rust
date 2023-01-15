use log::{error, info};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::{env, fs};
use std::path::{PathBuf, Path};

pub static PATH_TO_UPDATE_REPO_SCRIPT: &str = "/Users/juliankozlowski/Desktop/Studia/Rust/continous_integration_tool/bash_scripts/update_repo.sh";

pub fn read_commit_id(commit_path: &str) -> String {
    fs::read_to_string(commit_path).unwrap()
}

#[derive(Parser)]
pub struct DispatcherConfig {
    #[clap(long, default_value = "127.0.0.1:8888")]
    pub socket: SocketAddrV4,
    #[clap(long, default_value = "/Users/juliankozlowski/Desktop/Studia/Rust/continous_integration_tool/src/bin/test_repo_clone_runner")]
    pub repository_path: String,
}

fn send_massage(socket: &SocketAddrV4, request: Request) -> Response {
    match TcpStream::connect(socket) {
        Ok(mut stream) => {
            info!("Connected to socket: {:?}", socket);
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
    send_massage(socket, Request::CheckStatus)
}

pub fn update(socket: &SocketAddrV4, commit_path: &Path) -> Response {
    let commit_id = read_commit_id(commit_path.to_str().unwrap());
    info!("Updating dispatcher with commit id: {}", commit_id);
    send_massage(socket, Request::Update(commit_id))
}


#[derive(Serialize, Deserialize)]
pub enum Request {
    CheckStatus,
    Update(String),
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Ok,
    Error,
}

