use std::ffi::{c_double, CStr, CString, c_char};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, thread};
use std::time::Duration;
use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use telegram_drive::cloud::{Cloud, CloudError};
use telegram_drive::TelegramBackend;
use telegram_drive::virtual_file_system::VirtualFileSystem;

use telegram_drive_core::*;

const WAIT_TIMEOUT: f64 = 2.0;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    message_id: i64,
    content: Value
}

fn main() {

    println!("{:?}", TDApp::execute_query(&json!(
        {
            "@type": "setLogVerbosityLevel",
            "new_verbosity_level": 1
        }
    ).to_string()).unwrap());

    let mut fs = fs::File::open("fs.txt").unwrap();

    let mut fs_str = String::new();
    fs.read_to_string(&mut fs_str).unwrap();
    let vfs = serde_json::from_str::<VirtualFileSystem>(&fs_str).unwrap();

    let telegram_backend = TelegramBackend::new();
    println!("Backend has been created!");

    let mut cloud = Cloud::new(telegram_backend, Some(vfs));

    cloud.upload_file(Path::new(r"C:\Users\nikiy\Downloads\PLvs8_Kv0hU.jpg"), Path::new("fs://")).unwrap();

    println!("\n\n\n\nСкачивание\n\n\n\n");

    thread::sleep(Duration::new(5,0));

    cloud.download_file(Path::new(r"fs://PLvs8_Kv0hU")).expect("eewewgwgw");

    let fs_str = cloud.get_fs_json();

    fs::write("fs.txt", fs_str.as_bytes()).unwrap();


    loop {

    }
}


//Обновление =>> {"@client_id":1,"@type":"updateDeleteMessages","chat_id":-1001418440636,"from_cache":true,"is_permanent":false,"message_ids":[14568914944]}
