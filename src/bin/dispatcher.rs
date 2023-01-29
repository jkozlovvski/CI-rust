#[path = "../lib/common.rs"]
mod common;
#[path = "../lib/dispatcher_lib.rs"]
mod dispatcher_lib;

use clap::Parser;
use common::*;
use dispatcher_lib::*;
use std::sync::Arc;
use std::thread::spawn;

fn main() {
    env_logger::init();
    let config = DispatcherConfig::parse();
    let server = Arc::new(Server::new(config.socket));

    let cloned = server.clone();
    spawn(move || {
        redistributor(cloned);
    });

    loop {
        let (socket, _) = server.tcp_listener.accept().unwrap();
        let socket_cloned = socket.try_clone().unwrap();
        let server_cloned = server.clone();

        spawn(move || {
            handle_connection(socket_cloned, server_cloned);
        });
    }
}
