#[path ="../lib/repo_observer_lib.rs"]
mod repo_observer_lib;

use log::{error, info};
use repo_observer_lib::*;
use std::env;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    env_logger::init();
    let config = DispatcherConfig::build(env::args());
    let script_path = get_path_to_script();
    let scripts_dir_path = get_scripts_dir_path();
    env::set_current_dir(Path::new(&scripts_dir_path)).unwrap();
    let commit_path = Path::new(".commit_id");
    let dispatcher_connection = DispatcherConnection::new(config.socket);

    loop {
        let command_result = Command::new(&script_path)
            .arg(&config.repository_path)
            .arg(&config.socket.ip().to_string())
            .arg(&config.socket.port().to_string())
            .output();

        info!(
            "Running script in repo_observer, result: {:?}",
            command_result
        );

        match command_result {
            Ok(_) => {
                if commit_path.exists() {
                    if let Response::Error = dispatcher_connection.check_status() {
                        error!("Error while checking dispatcher status");
                        std::process::exit(1);
                    }

                    let commit_id = read_commit_id(commit_path.to_str().unwrap().to_string());
                    if let Response::Error = dispatcher_connection.update(commit_id) {
                        error!("Error while updating dispatcher");
                        std::process::exit(1);
                    }
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
