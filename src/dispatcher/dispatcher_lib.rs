use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet as Set};
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

pub struct DispatcherConfig {
    pub socket: SocketAddrV4,
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
        }
    }
}

#[derive(Debug)]
pub struct Server {
    pub tcp_listener: TcpListener,
    pub alive: AtomicBool,
    pub runners: Mutex<Vec<SocketAddrV4>>,
    pub dispatched_commits: Mutex<HashMap<String, String>>,
    pub pending_commits: Mutex<Set<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
enum Request {
    Status,
    Register(SocketAddrV4),
    Dispatch(String),
    Results((String, String)),
}

#[derive(Debug, Serialize, Deserialize)]
enum Response {
    Ok,
    Error(String),
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

pub fn runners_checker(server: Arc<Server>) {
    loop {
        if !server.alive.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        for (i, runner) in server.runners.lock().unwrap().iter().enumerate() {
            if let Err(_) = TcpStream::connect(runner) {
                info!(
                    "Runner {} is dead, deleting runner from the pool of available ones",
                    runner
                );
                server.runners.lock().unwrap().remove(i);
            }
        }
    }
}

pub fn redistributor(server: Arc<Server>) {
    loop {
        if !server.alive.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        for commit in server.pending_commits.lock().unwrap().iter() {
            dispatch_test(commit, server.clone());
            sleep(Duration::from_secs(5));
        }
    }
}

fn dispatch_test(commit: &String, server: Arc<Server>) {
    loop {
        for runner in server.runners.lock().unwrap().iter() {
            if let Ok(mut stream) = TcpStream::connect(runner) {
                let request = Request::Dispatch(commit.clone());
                serde_json::to_writer(&stream, &request).unwrap();
                stream.flush().unwrap();
                server
                    .dispatched_commits
                    .lock()
                    .unwrap()
                    .insert(commit.clone(), runner.to_string());
                server.pending_commits.lock().unwrap().remove(commit);
                break;
            }
        }
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
                serde_json::to_writer(&stream, &Response::Ok).unwrap();
                stream.flush().unwrap();
            }
            Ok(Request::Register(runner)) => {
                server.runners.lock().unwrap().push(runner);
                serde_json::to_writer(&stream, &Response::Ok).unwrap();
                stream.flush().unwrap();
            }
            Ok(Request::Dispatch(commit)) => {
                if server.runners.lock().unwrap().len() == 0 {
                    serde_json::to_writer(
                        &stream,
                        &Response::Error("No runners available".to_string()),
                    )
                    .unwrap();
                    stream.flush().unwrap();
                } else {
                    dispatch_test(&commit, server.clone());
                    server.pending_commits.lock().unwrap().insert(commit);
                    serde_json::to_writer(&stream, &Response::Ok).unwrap();
                    stream.flush().unwrap();
                }
            }
            Ok(Request::Results((commit_id, result))) => {
                server
                    .dispatched_commits
                    .lock()
                    .unwrap()
                    .insert(commit_id, result);
                serde_json::to_writer(&stream, &Response::Ok).unwrap();
                stream.flush().unwrap();
            }
            Err(_) => {
                break;
            }
        }
    }
}
