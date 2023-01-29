#[path = "../lib/test_runner_lib.rs"]
mod test_runner;

use clap::Parser;
use log::info;
use std::env;
use std::net::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::{sync::atomic::AtomicBool, thread::spawn};
use test_runner::*;

fn main() {
    env_logger::init();
    let dispatcher_alive = Arc::new(AtomicBool::new(true));
    let server = Arc::new(TestRunner::parse());
    let busy = Arc::new(AtomicBool::new(false));

    let binding = PathBuf::from(TEST_SCRIPT_PATH);
    let working_dir = binding.parent().unwrap();
    env::set_current_dir(working_dir).unwrap();

    info!("Working directory: {:?}", working_dir);

    let dispatcher_alive_cloned = dispatcher_alive.clone();
    let server_cloned = server.clone();
    spawn(move || {
        dispatcher_checker(server_cloned, dispatcher_alive_cloned);
    });

    let listener = TcpListener::bind(server.test_runner_socket).unwrap();
    send_socket_info(server.clone());

    loop {
        if !dispatcher_alive.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        let (socket, _) = listener.accept().unwrap();
        handle_connection(socket, busy.clone(), server.clone());
    }
}
