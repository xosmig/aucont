extern crate aucont_core as core;

use ::core::check::Check;

fn main() {
    let conts = ::core::list_dir_files(::core::CONTAINERS_DIR)
        .check("Error listing containers");
    for cont in conts {
        println!("{}", cont);
    }
}
