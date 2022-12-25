use serde::{Deserialize, Serialize};
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::{env, fs};

pub fn get_path_to_script() -> String {
    match env::var("UPDATE_SCRIPT_PATH") {
        Ok(path) => path,
        Err(err) => {
            println!("Error while getting path to script: {:?}", err);
            std::process::exit(1);
        }
    }
}

pub fn read_commit_id(commit_path: String) -> String {
    fs::read_to_string(commit_path).unwrap()
}

pub struct DispatcherConfig {
    pub socket: SocketAddrV4,
    pub repository_path: String,
}

impl DispatcherConfig {
    pub fn build(mut args: impl Iterator<Item = String>) -> DispatcherConfig {
        args.next();

        let dispatcher_host: (u8, u8, u8, u8) = match args.next() {
            Some(arg) => {
                let mut host = arg.split(".");
                (
                    host.next().unwrap().parse::<u8>().unwrap(),
                    host.next().unwrap().parse::<u8>().unwrap(),
                    host.next().unwrap().parse::<u8>().unwrap(),
                    host.next().unwrap().parse::<u8>().unwrap(),
                )
            }
            None => (127, 0, 0, 1),
        };

        let dispatcher_port = match args.next() {
            Some(arg) => arg.parse::<u16>().unwrap(),
            None => 8080,
        };

        let repository_path = match args.next() {
            Some(arg) => arg,
            None => String::from("/Users/juliankozlowski/Desktop/Studia/Rust/continous_integration_tool/src/repo_observer/test_repo_clone_obs")
        };

        DispatcherConfig {
            socket: SocketAddrV4::new(
                Ipv4Addr::new(
                    dispatcher_host.0,
                    dispatcher_host.1,
                    dispatcher_host.2,
                    dispatcher_host.3,
                ),
                dispatcher_port,
            ),
            repository_path,
        }
    }
}

pub struct DispatcherConnection {
    dispatcher_socket: SocketAddrV4,
}

impl DispatcherConnection {
    pub fn new(dispatcher_socket: SocketAddrV4) -> DispatcherConnection {
        DispatcherConnection { dispatcher_socket }
    }

    fn send_massage(&self, request: Request) -> Response {
        match TcpStream::connect(self.dispatcher_socket) {
            Ok(mut stream) => {
                serde_json::to_writer(&stream, &request).unwrap();
                stream.flush().unwrap();
                let mut deserializer =
                    serde_json::Deserializer::from_reader(stream.try_clone().unwrap());
                match Response::deserialize(&mut deserializer) {
                    Ok(_) => Response::Ok,
                    Err(err) => {
                        println!("Error while reading response: {:?}", err);
                        std::process::exit(1);
                    }
                }
            }
            Err(error) => {
                println!("Error: {:?}", error);
                std::process::exit(1);
            }
        }
    }

    pub fn check_status(&self) -> Response {
        self.send_massage(Request::CheckStatus)
    }

    pub fn update(&self, commit_id: String) -> Response {
        self.send_massage(Request::Update(commit_id))
    }
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
