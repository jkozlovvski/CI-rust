use repo_observer_lib::*;
use std::env;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use std::thread::sleep;

fn main() {
    let config = DispatcherConfig::build(env::args());
    let script_path = get_path_to_script();
    let commit_path = Path::new(".commit_id");

    loop {
        println!("{}", String::from("bash") + " " + &script_path);

        let command_result = Command::new(String::from("bash") + " " + &script_path)
                .arg(&config.repository_path)
                .arg(&config.socket.ip().to_string())
                .arg(&config.socket.port().to_string())
                .output();

        println!("{:?}", command_result);

        sleep(Duration::from_secs(5));

        // match command_result {
        //     Ok(output) => {
        //         if (commit_path.exists()) {
                    
        //         }
        //     },
        //     Err(err) => {
        //         println!("Error while executing script: {:?}", err);
        //         std::process::exit(1);
        //     }
        // }

    }
}
