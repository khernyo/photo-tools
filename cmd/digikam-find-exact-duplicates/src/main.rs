extern crate chrono;
extern crate dotenv;
extern crate rusqlite;

use std::env;
use std::path::Path;

use chrono::NaiveDateTime;
use dotenv::dotenv;
use rusqlite::{OpenFlags, Connection};

#[derive(Debug)]
pub struct Image {
    id: Option<i32>,
    album: Option<i32>,
    name: String,
    status: i32,
    category: i32,
    modification_date: Option<NaiveDateTime>,
    file_size: Option<i32>,
    unique_hash: Option<String>,
}

fn open_db() -> Connection {
    dotenv().ok();

    let d = env::var("DATABASE_DIR").expect("DATABASE_DIR must be set");
    let database_dir = Path::new(&d);
    let digikam_db = database_dir.join("digikam4.db");
    let recognition_db = database_dir.join("recognition.db");
    let thumbs_db = database_dir.join("thumbnails-digikam.db");
    let conn = Connection::open_with_flags(&digikam_db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect(&format!("Error connecting to {:?}", digikam_db));
    conn.execute("ATTACH ?1 AS recog", &[&recognition_db.to_str().unwrap()]).unwrap();
    conn.execute("ATTACH ?1 AS thumbs", &[&thumbs_db.to_str().unwrap()]).unwrap();
    conn
}

fn main() {
    let conn = open_db();
    println!("Images: {}", count_rows(&conn, "Images"));
    println!("Thumbnails: {}", count_rows(&conn, "thumbs.Thumbnails"));
    println!("Thumbnail filepaths: {}", count_rows(&conn, "thumbs.FilePaths"));
    println!("duplicate images: {}", duplicate_image_count(&conn));
    conn.close().unwrap();
}

fn count_rows(conn: &Connection, table: &str) -> u32 {
    let mut stmt = conn.prepare(&format!("SELECT COUNT(*) FROM {}", table)).unwrap();
    let result = stmt.query_map(&[], |row| {
        row.get::<_, u32>(0)
    }).unwrap();
    single(result).unwrap()
}

fn single<T: Iterator>(mut it: T) -> T::Item {
    let v = it.next().unwrap();
    assert!(it.next().is_none());
    v
}

fn duplicate_image_count(conn: &Connection) -> u32 {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM Images where id in (select id from Images group by uniqueHash having count(*) >= 2)").unwrap();
    let result = stmt.query_map(&[], |row| {
        row.get::<_, u32>(0)
    }).unwrap();
    single(result).unwrap()
}

fn duplicate_images(conn: &Connection) -> Vec<&Path> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM Images where id in (select id from Images group by uniqueHash having count(*) >= 2)").unwrap();
    let result = stmt.query_map(&[], |row| {
        row.get::<_, u32>(0)
    }).unwrap();
    single(result).unwrap();
    unimplemented!()
}

// select count(*) from Images where id in (select id from Images group by uniqueHash having count(*) >= 2)
// select path from thumbs.FilePaths where thumbId in (select thumbId from thumbs.FilePaths group by thumbId having count(*) >= 2)
