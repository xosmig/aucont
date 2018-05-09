extern crate aucont_core as core;
extern crate clap;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use ::std::fs;
use ::core::check::Check;


#[derive(Serialize, Deserialize, Debug)]
struct Config {
    image_path: String,
    cmd: String,
    args: Vec<String>,
    replica_count: u32,
    output_dir_path: String,
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
    let _config: Config = serde_json::from_reader(config_file)
        .check("ERROR reading or parsing config");



}
