mod lib;

use clap::Parser;
use log::{error, info};
use std::env;
use std::net::*;
use std::path::Path;
use std::sync::Arc;
use std::{sync::atomic::AtomicBool, thread::spawn};
use lib::test_runner_lib::*;
use lib::common::*;

fn main() {
    env_logger::init();
    let dispatcher_alive = Arc::new(AtomicBool::new(true));
    let server = Arc::new(TestRunner::parse());
    let busy = Arc::new(AtomicBool::new(false));

    let working_dir = Path::new(&scripts_repository);
    if let Err(err) = env::set_current_dir(working_dir) {
        error!("Error while setting working directory: {:?}", err);
        std::process::exit(1);
    }

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
