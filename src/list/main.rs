extern crate aucont;

fn main() {
    let conts = ::aucont::list_dir_files(::aucont::CONTAINERS_DIR)
        .expect("Error listing containers");
    for cont in conts {
        println!("{}", cont);
    }
}
