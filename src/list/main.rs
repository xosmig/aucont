extern crate aucont;

use ::aucont::check::Check;

fn main() {
    let conts = ::aucont::list_dir_files(::aucont::CONTAINERS_DIR)
        .check("Error listing containers");
    for cont in conts {
        println!("{}", cont);
    }
}
