mod common;
use common::*;

use log::{info, error};
use clap::Parser;
use std::net::SocketAddrV4;
use std::thread::sleep;
use std::sync::Arc;
use std::time::Duration;
use std::sync::atomic::AtomicBool;
use std::net::TcpStream;
use serde::Deserialize;
use std::process::Command;
use std::fs::File;
use std::io::prelude::*;

pub static TEST_SCRIPT_PATH: &str = "/Users/juliankozlowski/Desktop/Studia/Rust/continous_integration_tool/bash_scripts/test_runner.sh";

#[derive(Parser, Debug)]
pub struct TestRunner {
    #[clap(long, default_value = "127.0.0.1:8888")]
    pub dispatcher_socket: SocketAddrV4,
    #[clap(long, default_value = "127.0.0.1:10000")]
    pub test_runner_socket: SocketAddrV4,
    #[clap(long, default_value = "/Users/juliankozlowski/Desktop/Studia/Rust/continous_integration_tool/src/bin/test_repo_clone_runner")]
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

pub fn handle_connection(mut stream: TcpStream, busy: bool, server: Arc<TestRunner>) {
    let mut deserializer = serde_json::Deserializer::from_reader(stream.try_clone().unwrap());
    match Request::deserialize(&mut deserializer) {
        Ok(request) => {
            match request {
                Request::Status => {
                    if busy {
                        serde_json::to_writer(&stream, &Response::Error("Busy".to_string())).unwrap();
                    }
                    else {
                        serde_json::to_writer(&stream, &Response::Ok).unwrap();
                    }
                }
                Request::Dispatch(commid_id) => {
                    info!("Dispatching test with id: {}", commid_id);
                    run_tests(server);
                    serde_json::to_writer(&stream, &Response::Ok).unwrap();
                }
                _ => {
                    error!("Invalid request");
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            error!("Error while reading request: {:?}", err);
            std::process::exit(1);
        }
    }
}

pub fn send_socket_info(server: Arc<TestRunner>) {
    info!("Sending socket info  about socket to dispatcher");
    match send_massage(&server.dispatcher_socket, Request::Register(server.test_runner_socket)) {
        Response::Ok => {
            info!("Socket info sent");
        }
        Response::Error(err) => {
            error!("Error while sending socket info: {:?}", err);
            std::process::exit(1);
        }
    }
}

fn send_tests_results(server: Arc<TestRunner>, results: String, commit_id: String) {
    info!("Sending test results to dispatcher");
    match send_massage(&server.dispatcher_socket, Request::Results((commit_id, results))) {
        Response::Ok => {
            info!("Test results sent");
        }
        Response::Error(err) => {
            error!("Error while sending test results: {:?}", err);
            std::process::exit(1);
        }
    }
}

pub fn run_tests(server: Arc<TestRunner>) {
    // updating repo
    let output = Command::new(TEST_SCRIPT_PATH)
        .arg(server.repository_path.clone())
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