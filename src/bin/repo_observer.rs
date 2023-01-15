#[path ="../lib/repo_observer_lib.rs"]
mod repo_observer_lib;
#[path ="../lib/common.rs"]
mod common;

use clap::Parser;
use log::{error, info};
use repo_observer_lib::*;
use common::*;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    env_logger::init();
    let config = RepoObserverConfig::parse();
    let (dispatcher_socket, repository_path) = (config.socket, config.repository_path);

    let script_path = PATH_TO_UPDATE_REPO_SCRIPT;
    let binding = PathBuf::from(script_path);
    let working_dir = binding.parent().unwrap();
    env::set_current_dir(working_dir).unwrap();
    let commit_path = Path::new(".commit_id");

    info!("Working directory: {:?}", working_dir);
    info!("Script path: {:?}", script_path);
    info!("Commit path: {:?}", commit_path);

    loop {
        let command_result = Command::new(&script_path)
            .arg(&repository_path)
            .output();

        info!(
            "Running script in repo_observer, result: {:?}",
            command_result
        );

        match command_result {
            Ok(_) => {
                if commit_path.exists() {
                    info!("Found commit path");
                    check_status(&dispatcher_socket);
                    update(&dispatcher_socket, commit_path);
                }
            }
            Err(err) => {
                error!("Error while running script: {:?}", err);
                std::process::exit(1);
            }
        }

        sleep(Duration::from_secs(5));
    }
}
