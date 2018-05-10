extern crate aucont_core as core;

use ::core::check::Check;
use ::std::fs;

fn main() {
    // create dir if it doesn't exist
    fs::create_dir_all(::core::CONTAINERS_DIR)
        .check("Error accessing containers directory");
    let conts = ::core::list_dir_files(::core::CONTAINERS_DIR)
        .check("Error listing containers");
    for cont in conts {
        println!("{}", cont);
    }
}
