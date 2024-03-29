extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate structopt;
extern crate walkdir;

use std::path::Path;
use std::process::Command;

use regex::Regex;
use structopt::StructOpt;

lazy_static! {
    static ref FILE_NUMBER_RE: Regex = Regex::new(r#"^.*_([0-9]+)\.[^.]+$"#).unwrap();
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(name = "src-dir")]
    src_dir: String,

    #[structopt(name = "dst-dir")]
    dst_base_dir: String,
}

fn main() {
    let opts = Opts::from_args();
    get(&opts.src_dir, &opts.dst_base_dir);
}

fn get(src_dir: &str, dst_base_dir: &str) {
    for entry in WalkDir::new(src_dir) {
        if let Ok(e) = entry {
            let path = e.path();
            if e.is_file() {
                if e.has_extension_ignorecase(&["jpg", "cr2", "mp4"]) {
                    let parent = path.parent().unwrap();
                    let parent_name = parent.file_name().unwrap().to_string_lossy();
                    let parent_parent = path.parent().unwrap().parent().unwrap();
                    let parent_parent_name = parent_parent.file_name().unwrap().to_string_lossy();
                    let file_name = e
                        .file_name()
                        .to_str()
                        .unwrap_or_else(|| panic!("Can't handle filename: {:?}", e.file_name()));

                    assert_eq!(
                        parent_parent_name, "DCIM",
                        "Unexpected directory structure: {:?}",
                        e
                    );
                    assert!(
                        parent_name.ends_with("CANON"),
                        "Could not determine dir number: {:?}",
                        e
                    );

                    let dir_number = &parent_name.as_ref().drop_tail("CANON");
                    assert!(
                        dir_number.parse::<u32>().is_ok(),
                        "Invalid dir number: {} from {:?}",
                        dir_number,
                        e
                    );

                    let file_number_from_fname = FILE_NUMBER_RE
                        .captures_iter(file_name)
                        .next()
                        .unwrap_or_else(|| {
                            panic!("Could not determine file number: {}", file_name)
                        })[1]
                        .parse::<u32>();
                    let file_number_from_exif = exif_info("FileIndex", path).parse::<u32>();
                    let file_number = file_number_from_exif
                        .clone()
                        .or_else(|_| file_number_from_fname.clone())
                        .unwrap_or_else(|_| {
                            panic!(
                                "Could not determine file number: {:?} {:?}",
                                file_number_from_fname, file_number_from_exif
                            )
                        });

                    let create_date = exif_info("CreateDate", path);
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
                        assert!(Command::new("cp")
                            .args(&[path.to_str().unwrap(), target.to_str().unwrap()])
                            .status()
                            .unwrap()
                            .success());
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

fn exif_info(field: &str, path: &std::path::Path) -> String {
    let result = Command::new("exiftool")
        .args(&[&format!("-{}", field), "-b", path.to_str().unwrap()])
        .output()
        .expect("Error while running exiftool");
    assert!(result.status.success());

    String::from_utf8(result.stdout).unwrap()
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
