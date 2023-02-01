use super::common;

use clap::Parser;
use common::{check_status, send_massage, Request, Response};
use log::{error, info};
use serde::Deserialize;
use std::{
    fs::File, io::prelude::*, net::SocketAddrV4, net::TcpStream, process::Command,
    sync::atomic::AtomicBool, sync::Arc, thread::sleep, time::Duration,
};

#[derive(Parser, Debug)]
pub struct TestRunner {
    #[clap(long, default_value = "127.0.0.1:8888")]
    pub dispatcher_socket: SocketAddrV4,
    #[clap(long, default_value = "127.0.0.1:10030")]
    pub test_runner_socket: SocketAddrV4,
    #[clap(
        long,
        default_value = "/Users/juliankozlowski/Desktop/Studia/Rust/continous_integration_tool/src/bin/test_repo_clone_runner"
    )]
    pub repository_path: String,
}

pub fn dispatcher_checker(server: Arc<TestRunner>, dispatcher_alive: Arc<AtomicBool>) {
    loop {
        match check_status(&server.dispatcher_socket) {
            Response::Ok => {
                info!("Dispatcher is alive");
            }
            Response::Error(_) => {
                info!("Dispatcher is down");
                dispatcher_alive.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
        sleep(Duration::from_secs(5));
    }
}

pub fn handle_connection(stream: TcpStream, busy: Arc<AtomicBool>, server: Arc<TestRunner>) {
    let mut deserializer = serde_json::Deserializer::from_reader(stream.try_clone().unwrap());
    match Request::deserialize(&mut deserializer) {
        Ok(request) => match request {
            Request::Status => {
                if busy.load(std::sync::atomic::Ordering::Relaxed) {
                    serde_json::to_writer(&stream, &Response::Error("Busy".to_string())).unwrap();
                } else {
                    serde_json::to_writer(&stream, &Response::Ok).unwrap();
                }
            }
            Request::Dispatch(commid_id) => {
                info!("Dispatching test with id: {}", commid_id);
                run_tests(server, commid_id);
                serde_json::to_writer(&stream, &Response::Ok).unwrap();
            }
            _ => {
                error!("Invalid request");
                std::process::exit(1);
            }
        },
        Err(_) => (),
    }
}

pub fn send_socket_info(server: Arc<TestRunner>) {
    info!("Sending socket info  about socket to dispatcher");
    match send_massage(
        &server.dispatcher_socket,
        Request::Register(server.test_runner_socket),
    ) {
        Response::Ok => {
            info!("Socket info sent");
        }
        Response::Error(err) => {
            error!("Error while sending socket info: {:?}", err);
        }
    }
}

pub fn run_tests(server: Arc<TestRunner>, commit_id: String) {
    let output = Command::new("./test_runner.sh")
        .arg(server.repository_path.clone())
        .arg(commit_id)
        .output()
        .expect("failed to execute script");

    info!("Updating repo: {}", String::from_utf8_lossy(&output.stdout));

    let testing_path_argument = format!("--manifest-path={}/Cargo.toml", server.repository_path);
    info!("Running tests: {}", testing_path_argument);

    let output = Command::new("cargo")
        .arg("test")
        .arg(testing_path_argument)
        .output()
        .expect("failed to execute script");

    info!("Running tests: {}", String::from_utf8_lossy(&output.stdout));

    let mut file = match File::create("TestResults.txt") {
        Ok(file) => file,
        Err(err) => {
            error!("Error while creating file: {:?}", err);
            std::process::exit(1);
        }
    };

    match file.write_all(&output.stdout) {
        Ok(_) => info!("Saved test results to file"),
        Err(err) => error!("Error while saving test results to file: {:?}", err),
    }
}
