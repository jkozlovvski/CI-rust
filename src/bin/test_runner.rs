#[path = "../lib/common.rs"]
mod common;

#[path = "../lib/test_runner_lib.rs"]
mod test_runner;

use std::{sync::atomic::AtomicBool, thread::spawn};
use std::sync::Arc;
use clap::Parser;
use common::*;
use test_runner::*;
use log::info;

fn main() {
    env_logger::init();
    let dispatcher_alive = Arc::new(AtomicBool::new(true));
    let dispatcher_socket= Arc::new(DispatcherConfig::parse().socket);


    let dispatcher_alive_cloned = dispatcher_alive.clone();
    let dispatcher_socket_cloned = dispatcher_socket.clone();
    spawn (move || {
        dispatcher_checker(dispatcher_socket_cloned, dispatcher_alive_cloned);
    });
    
    loop {
        if !dispatcher_alive.load(std::sync::atomic::Ordering::Relaxed) {
            
        }
    }

}
