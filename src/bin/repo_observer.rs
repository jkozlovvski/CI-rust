mod lib;

use clap::Parser;
use log::{error, info};
use lib::{
    common::*, repo_observer_lib::*
};

use std::{
    env, process::Command, thread::sleep, time::Duration, path::Path
};

fn main() {
    env_logger::init();
    let config = RepoObserverConfig::parse();
    let (dispatcher_socket, repository_path) = (config.socket, config.repository_path);
    
    let working_dir = Path::new(&scripts_repository);
    if let Err(err) = env::set_current_dir(working_dir) {
        error!("Error while setting working directory: {:?}", err);
        std::process::exit(1);
    }

    let commit_path = Path::new(".commit_id");
    let script_path = Path::new("./update_repo.sh");

    info!("Working directory: {:?}", working_dir);
    info!("Script path: {:?}", script_path);
    info!("Commit path: {:?}", commit_path);

    loop {
        let command_result = Command::new(script_path).arg(&repository_path).output();

        info!(
            "Running script in repo_observer, result: {:?}",
            command_result
        );

        match command_result {
            Ok(_) => {
                if commit_path.exists() {
                    info!("Found commit path");
                    check_status(&dispatcher_socket);
                    dispatch(&dispatcher_socket, commit_path);
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
