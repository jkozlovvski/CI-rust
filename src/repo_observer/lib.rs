use std::{env, fs};
use std::net::{SocketAddrV4, Ipv4Addr};

pub fn get_path_to_script() -> String {
    match env::var("UPDATE_SCRIPT_PATH") {
        Ok(path) => path,
        Err(err) => {
            println!("Error while getting path to script: {:?}", err);
            std::process::exit(1);
        }
    }
}

pub fn read_commit_id(commit_path: String) -> String {
    fs::read_to_string(commit_path).unwrap()
}

pub struct DispatcherConfig {
    pub socket: SocketAddrV4,
    pub repository_path: String
}

impl DispatcherConfig {
    pub fn build(mut args: impl Iterator<Item = String>) -> DispatcherConfig {
        args.next();

        let dispatcher_host: (u8, u8, u8, u8) = match args.next() {
            Some(arg) => {
                let mut host = arg.split(".");
                (host.next().unwrap().parse::<u8>().unwrap(), host.next().unwrap().parse::<u8>().unwrap(), host.next().unwrap().parse::<u8>().unwrap(), host.next().unwrap().parse::<u8>().unwrap())
            },
            None => (127, 0, 0, 1)
        };

        let dispatcher_port = match args.next() {
            Some(arg) => arg.parse::<u16>().unwrap(),
            None => 8080 
        };
            
        let repository_path = match args.next() {
            Some(arg) => arg,
            None => String::from("/Users/juliankozlowski/Desktop/Studia/Rust/continous_integration_tool/src/repo_observer/test_repo_clone_obs")
        };

        DispatcherConfig {socket: SocketAddrV4::new(Ipv4Addr::new(dispatcher_host.0, dispatcher_host.1, dispatcher_host.2, dispatcher_host.3), dispatcher_port), repository_path}
    }
}