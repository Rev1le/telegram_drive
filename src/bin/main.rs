use telegram_drive::telegram_backend::TelegramBackend;

fn main() {

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let telegram_backend = TelegramBackend::new().await;

        loop {

        }
    });

    // let opt_vfs_load = match fs::File::open("fs.txt") {
    //     Ok(mut f) => {
    //         let mut fs_str = String::new();
    //         f.read_to_string(&mut fs_str).unwrap();
    //         Some(serde_json::from_str::<VirtualFileSystem>(&fs_str).unwrap())
    //     },
    //     Err(_) => None
    // };
    //
    // let telegram_backend = telegram_backend::TelegramBackend::new();
    // println!("Backend has been created!");
    //
    // let mut cloud = Cloud::new(telegram_backend, opt_vfs_load);
    //
    // //cloud.upload_file(Path::new(r"C:\Users\nikiy\Downloads\PLvs8_Kv0hU.jpg"), Path::new("fs://")).unwrap();
    //
    // println!("\n\n\n\nСкачивание\n\n\n\n");
    //
    // thread::sleep(Duration::new(5,0));
    //
    // cloud.download_file(Path::new(r"fs://PLvs8_Kv0hU")).expect("eewewgwgw");
    //
    // let fs_str = cloud.get_fs_json();
    //
    // fs::write("fs.txt", fs_str.as_bytes()).unwrap();

}


//Обновление =>> {"@client_id":1,"@type":"updateDeleteMessages","chat_id":-1001418440636,"from_cache":true,"is_permanent":false,"message_ids":[14568914944]}
