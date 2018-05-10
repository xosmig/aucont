#[macro_use(shell, sudo)]
extern crate aucont_core as core;
extern crate aucont_lib_container as container;

extern crate clap;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate scopeguard;

use ::std::{fs, process, env};
use ::std::str::FromStr;
use ::std::net::Ipv4Addr;
use ::core::check::Check;
use ::std::thread::{self, JoinHandle};
use ::core::getpid;
use ::container::factory::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    container_image_path: String,
    cmd: String,
    args: Vec<String>,
    replica_count: u32,
    output_dir_path: String,  // TODO
}

fn start_replica(config: Config, replica_id: u32, replica_ip: Ipv4Addr, bridge: String)
                 -> JoinHandle<container::Result<()>> {
    thread::spawn(move || {
        let output_path = format!("{}/output_{}.txt", config.output_dir_path, replica_id);
        for attempt in 0..10 {
            let config = config.clone();
            let container = ContainerFactory::new_container(ContainerConfig {
                daemonize: true,
                image_path: config.container_image_path,
                cmd: config.cmd,
                cmd_args: config.args,
                net: Some(NetworkConfig {
                    cont_addr: replica_ip,
                    host_addr: None,
                    host_bridge: Some(bridge.clone()),
                }),
                cpu_perc: None,
                environment: vec![
                    ("REPLICA_IX".to_string(), replica_id.to_string()),
                    ("REPLICA_COUNT".to_string(), config.replica_count.to_string()),
                    ("RESTART_NUMBER".to_string(), attempt.to_string()),
                ],
                redirect_stdout: Some(output_path.clone()),
                redirect_stderr: Some(output_path.clone()),
                ..Default::default()
            })?;

            let code = container.wait_and_clear()?;
            if code == 0 { return Ok(()); }
        }
        Err(container::Error::simple("Maximal number of attempts exceeded"))
    })
}


fn main() {
    let matches = clap::App::new("aucont_cluster")
        .version("0.1")
        .about("Orchestration tool for aucont containers.")
        .arg(clap::Arg::with_name("config")
            .index(1)
            .required(true)
            .value_name("CONFIG_FILE")
            .help("Config file path"))
        .get_matches();

    let config_path = matches.value_of("config").unwrap();

    let config_file = fs::File::open(config_path).check("ERROR opening config file");
    let config: Config = serde_json::from_reader(config_file)
        .check("ERROR reading or parsing config");

    let ips: Vec<_> = (0..config.replica_count).map(|replica_id| {
        let ip = format!("10.0.17.{}", replica_id + 1);
        env::set_var(format!("REPLICA_{}_IP", replica_id), &ip);
        Ipv4Addr::from_str(&ip).unwrap()
    }).collect();

    let pid = getpid();
    let bridge = format!("auc{}br", pid);
    sudo!("ip", "link", "add", &bridge, "type", "bridge").check("Error setting up network bridge");
    defer! {{
        sudo!("ip", "link", "del", &bridge).log_error("Error deleting network bridge");
    }};
    sudo!("ip", "link", "set", &bridge, "up").check("Error setting up network bridge");
    defer! {{
        sudo!("ip", "link", "set", &bridge, "down").log_error("Error shutting down network bridge");
    }};

    let threads: Vec<_> = (0..config.replica_count).map(|replica_id| {
        start_replica(config.clone(), replica_id, ips[replica_id as usize], bridge.clone())
    }).collect();

    let mut failed = false;
    for thread in threads {
        if let None = thread.join().unwrap().log_error("Error in a replica manager") {
            failed = true;
        }
    }

    if failed {
        process::exit(1);
    }
}
