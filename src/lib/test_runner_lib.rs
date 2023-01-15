mod common;
use common::*;

use log::info;
use std::net::SocketAddrV4;
use std::thread::sleep;
use std::sync::Arc;
use std::time::Duration;
use std::sync::atomic::AtomicBool;

pub struct TestRunnerServer {
    pub socket: SocketAddrV4,
}

pub fn dispatcher_checker(dispatcher_socket: Arc<SocketAddrV4>, dispatcher_alive: Arc<AtomicBool>) {
    loop {
        match check_status(dispatcher_socket.as_ref()) {
            Response::Ok => {
                info!("Dispatcher is alive");
                dispatcher_alive.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            Response::Error(_) => {
                info!("Dispatcher is down");
                dispatcher_alive.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
        sleep(Duration::from_secs(5));
    }
}