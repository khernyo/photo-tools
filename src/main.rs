extern crate clap;

use clap::{App,Arg};

fn main() {
    let arg_src_dir = Arg::with_name("src-dir")
        .help("Dir to copy photos from")
        .index(1);

    let matches = App::new("photo-tools")
        .arg(arg_src_dir.clone())
        .get_matches();

    let src_dir = matches.value_of(arg_src_dir.b.name).unwrap();

}
