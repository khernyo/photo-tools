extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate walkdir;

use std::path::Path;
use std::process::Command;
use clap::{App, Arg};
use regex::Regex;

lazy_static! {
    static ref FILE_NUMBER_RE: Regex = Regex::new(r#"^.*_([0-9]+)\.[^.]+$"#).unwrap();
}

fn main() {
    let arg_src_dir = Arg::with_name("src-dir")
        .help("Dir to copy photos from")
        .required(true)
        .index(1);
    let arg_dst_dir = Arg::with_name("dst-dir")
        .help("Dir to copy photos from")
        .required(true)
        .index(2);

    let matches = App::new("photo-tools")
        .arg(arg_src_dir.clone())
        .arg(arg_dst_dir.clone())
        .get_matches();

    let src_dir = matches.value_of(arg_src_dir.b.name).unwrap();
    let dst_base_dir = matches.value_of(arg_dst_dir.b.name).unwrap();

    for entry in WalkDir::new(src_dir) {
        if let Ok(e) = entry {
            let path = e.path();
            if e.is_file() {
                if e.has_extension_ignorecase(&["jpg", "cr2"]) {
                    let file_name = e.file_name().to_string_lossy();
                    let parent = path.parent().unwrap();
                    let parent_name = parent.file_name().unwrap().to_string_lossy();
                    let parent_parent = path.parent().unwrap().parent().unwrap();
                    let parent_parent_name = parent_parent.file_name().unwrap().to_string_lossy();

                    assert_eq!(
                        parent_parent_name,
                        "DCIM",
                        "Unexpected directory structure: {:?}",
                        e
                    );
                    assert!(
                        parent_name.ends_with("CANON"),
                        "Could not determine dir number: {:?}",
                        e
                    );

                    let dir_number = &parent_name.as_ref().drop_tail("CANON");
                    let file_number = &FILE_NUMBER_RE
                        .captures_iter(file_name.as_ref())
                        .next()
                        .unwrap()
                        [1];

                    assert!(
                        dir_number.parse::<u32>().is_ok(),
                        format!("Invalid dir number: {} from {:?}", dir_number, e)
                    );
                    assert!(
                        file_number.parse::<u32>().is_ok(),
                        format!("Invalid file number: {} from {:?}", file_number, e)
                    );

                    let create_date_output = Command::new("exiftool")
                        .args(&["-CreateDate", "-b", path.to_str().unwrap()])
                        .output()
                        .unwrap();
                    assert!(create_date_output.status.success());

                    let create_date = String::from_utf8(create_date_output.stdout).unwrap();
                    let dst_extension = path.extension().unwrap().to_string_lossy().to_lowercase();
                    let target_filename = format!(
                        "{}_{}-{}.{}",
                        create_date.replace(':', "-").replace(' ', "_"),
                        dir_number,
                        file_number,
                        dst_extension
                    );
                    let dst_dir = format!(
                        "{}/{}",
                        dst_base_dir,
                        target_filename.split('_').next().unwrap()
                    );
                    let target_s = format!("{}/{}", dst_dir, target_filename);
                    let target = Path::new::<str>(target_s.as_ref());

                    if target.exists() {
                        let done = Command::new("cmp")
                            .args(&["-s", path.to_str().unwrap(), target.to_str().unwrap()])
                            .status()
                            .unwrap()
                            .success();
                        if done {
                            println!("{:?} is already done", path);
                        } else {
                            panic!("{:?} and {:?} differ. Giving up!", path, target);
                        }
                    } else {
                        println!("Copying {:?} to {:?}", path, target);
                        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                        assert!(
                            Command::new("cp")
                                .args(&[path.to_str().unwrap(), target.to_str().unwrap()])
                                .status()
                                .unwrap()
                                .success()
                        );
                    }
                } else {
                    println!("E? {:?}", path);
                }
            }
        } else {
            panic!("{:?}", entry);
        }
    }
}

use walkdir::{DirEntry, WalkDir};

trait ShellDirEntryTools {
    fn is_file(&self) -> bool;
    fn has_extension_ignorecase(&self, extensions: &[&str]) -> bool;
}

impl ShellDirEntryTools for DirEntry {
    fn is_file(&self) -> bool {
        Path::new(self.path()).is_file()
    }

    fn has_extension_ignorecase(&self, extensions: &[&str]) -> bool {
        let extension = self.path().extension();
        if let Some(ext) = extension.map(|e| e.to_string_lossy().to_lowercase()) {
            extensions.iter().any(|e| *e.to_lowercase() == ext)
        } else {
            false
        }
    }
}

trait ShellStringTools {
    fn drop_tail(&self, tail: &str) -> Self;
}

impl<'a> ShellStringTools for &'a str {
    fn drop_tail<'b>(&self, tail: &'b str) -> Self {
        if self.ends_with(tail) {
            &self[..(self.len() - tail.len())]
        } else {
            self
        }
    }
}
