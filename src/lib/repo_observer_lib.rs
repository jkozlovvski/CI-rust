mod common;
use common::*;

use log::info;
use clap::Parser;

use std::{
    fs,
    net::SocketAddrV4,
    path::Path,
};

pub static PATH_TO_UPDATE_REPO_SCRIPT: &str = "/Users/juliankozlowski/Desktop/Studia/Rust/continous_integration_tool/bash_scripts/update_repo.sh";

pub fn read_commit_id(commit_path: &str) -> String {
    fs::read_to_string(commit_path).unwrap()
}

#[derive(Parser)]
pub struct RepoObserverConfig {
    #[clap(long, default_value = "127.0.0.1:8888")]
    pub socket: SocketAddrV4,
    #[clap(long, default_value = "/Users/juliankozlowski/Desktop/Studia/Rust/continous_integration_tool/src/bin/test_repo_clone_runner")]
    pub repository_path: String,
}

pub fn update(socket: &SocketAddrV4, commit_path: &Path) -> Response {
    let commit_id = read_commit_id(commit_path.to_str().unwrap());
    info!("Updating dispatcher with commit id: {}", commit_id);
    send_massage(socket, Request::Update(commit_id))
}

