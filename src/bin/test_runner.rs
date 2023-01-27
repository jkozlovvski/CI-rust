#[path = "../lib/common.rs"]
mod common;

#[path = "../lib/test_runner_lib.rs"]
mod test_runner;

use std::{sync::atomic::AtomicBool, thread::spawn};
use std::sync::Arc;
use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::net::*;
use common::*;
use test_runner::*;
use log::info;

fn main() {
    env_logger::init();
    let dispatcher_alive = Arc::new(AtomicBool::new(true));
    let server = Arc::new(TestRunner::parse());
    let busy = false;

    let binding = PathBuf::from(TEST_SCRIPT_PATH);
    let working_dir = binding.parent().unwrap();
    env::set_current_dir(working_dir).unwrap();

    info!("Working directory: {:?}", working_dir);

    // let dispatcher_alive_cloned = dispatcher_alive.clone();
    // let server_cloned = server.clone();
    // spawn (move || {
    //     dispatcher_checker(server_cloned, dispatcher_alive_cloned);
    // });
    
    // let listener = TcpListener::bind(server.test_runner_socket).unwrap();
    // send_socket_info(server);

    // loop {
        // if dispatcher_alive.load(std::sync::atomic::Ordering::Relaxed) {
        //    break; 
        // }
        run_tests(server.clone());
        
        // let (socket, _) = listener.accept().unwrap();
    // }

}
