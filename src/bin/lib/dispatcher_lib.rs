use super::common;
use log::info;
use serde::Deserialize;

use common::{
    send_massage, Request, Response
};

use std::{
    collections::{HashMap, HashSet as Set},
    io::Write,
    net::SocketAddrV4,
    net::TcpListener,
    net::TcpStream,
    sync::atomic::AtomicBool,
    sync::Arc,
    sync::Mutex,
    thread::sleep,
    time::Duration,
};

#[derive(Debug)]
pub struct Server {
    pub tcp_listener: TcpListener,
    pub alive: AtomicBool,
    pub runners: Mutex<Vec<SocketAddrV4>>,
    pub dispatched_commits: Mutex<HashMap<String, String>>,
    pub pending_commits: Mutex<Set<String>>,
}

impl Server {
    pub fn new(socket: SocketAddrV4) -> Server {
        let tcp_listener = TcpListener::bind(socket).unwrap();
        Server {
            tcp_listener,
            alive: AtomicBool::new(true),
            runners: Mutex::new(Vec::new()),
            dispatched_commits: Mutex::new(HashMap::new()),
            pending_commits: Mutex::new(Set::new()),
        }
    }
}

pub fn redistributor(server: Arc<Server>) {
    loop {
        if !server.alive.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        let mut dead_runners = Vec::new();

        for runner in server.runners.lock().unwrap().iter_mut() {
            match TcpStream::connect(runner.clone()) {
                Ok(mut stream) => {
                    info!("Runner {} is alive", runner);
                    let mut dispatched_commits = server.dispatched_commits.lock().unwrap();
                    let mut pending_commits = server.pending_commits.lock().unwrap();
                    let mut commit_to_delete = Vec::new();

                    for commit in pending_commits.iter() {
                        if dispatched_commits.contains_key(commit) {
                            continue;
                        }

                        info!("Dispatching commit: {}", commit);
                        serde_json::to_writer(&stream, &Request::Dispatch(commit.clone()))
                            .unwrap();
                        stream.flush().unwrap();

                        let mut deserializer =
                            serde_json::Deserializer::from_reader(stream.try_clone().unwrap());
                        let response: Response = Response::deserialize(&mut deserializer).unwrap();

                        match response {
                            Response::Ok => {
                                dispatched_commits.insert(commit.clone(), runner.to_string());
                                commit_to_delete.push(commit.clone());
                            }
                            Response::Error(_) => {
                                info!("Runner {} is busy", runner);
                                break;
                            }
                        }
                    }

                    for commit in commit_to_delete {
                        pending_commits.remove(&commit);
                    }
                }
                Err(_) => {
                    info!("Runner {} is dead", runner);
                    dead_runners.push(runner.clone());
                }
            }
        }

        for dead_runner in dead_runners.iter() {
            info!("Removing dead runner: {}", dead_runner);
            server.runners.lock().unwrap().retain(|runner| runner != dead_runner);
        }
        
        sleep(Duration::from_secs(2));
    }
}

pub fn handle_connection(mut stream: TcpStream, server: Arc<Server>) {
    let mut deserializer = serde_json::Deserializer::from_reader(stream.try_clone().unwrap());
    loop {
        if !server.alive.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        match Request::deserialize(&mut deserializer) {
            Ok(Request::Status) => {
                info!("Status request");
                serde_json::to_writer(&stream, &Response::Ok).unwrap();
                stream.flush().unwrap();
            }
            Ok(Request::Update(commit)) => {
                info!("Updating commit {}", commit);
                serde_json::to_writer(&stream, &Response::Ok).unwrap();
            }
            Ok(Request::Register(runner)) => {
                info!("Registering runner {}", runner);
                server.runners.lock().unwrap().push(runner);
                serde_json::to_writer(&stream, &Response::Ok).unwrap();
                stream.flush().unwrap();
            }
            Ok(Request::Dispatch(commit)) => {
                info!("Adding commit to dispatched {}", commit);
                server.pending_commits.lock().unwrap().insert(commit);
                serde_json::to_writer(&stream, &Response::Ok).unwrap();
                stream.flush().unwrap();
            }
            Err(_) => {
                break;
            }
        }
    }
}
