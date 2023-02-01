use super::common;

use common::{
    send_massage, Request, Response
};
use std::{
    fs, net::SocketAddrV4, path::Path,
};
use log::info;
use clap::Parser;

pub fn read_commit_id(commit_path: &str) -> String {
    fs::read_to_string(commit_path).unwrap()
}

#[derive(Parser)]
pub struct RepoObserverConfig {
    #[clap(long, default_value = "127.0.0.1:8888")]
    pub socket: SocketAddrV4,
    #[clap(long, default_value = "../src/bin/test_repo_clone_obs")]
    pub repository_path: String,
}

pub fn dispatch(socket: &SocketAddrV4, commit_path: &Path) -> Response {
    let commit_id = read_commit_id(commit_path.to_str().unwrap());
    info!("Updating dispatcher with commit id: {}", commit_id);
    send_massage(socket, Request::Dispatch(commit_id))
}

