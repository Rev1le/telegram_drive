use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use telegram_drive::cloud::Cloud;
use telegram_drive::telegram_backend::TelegramBackend;
use telegram_drive::virtual_file_system::{FSOption, Metadata, VFSFile, VFSFolder, VirtualFileSystem};
use telegram_drive::cloud_backend::AsyncCloudBackend;
use telegram_drive::virtual_file_system::FileSystemNode::{File, Folder};
use telegram_drive_file::Options;
use std::io::Read;

fn main() {

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {

        let cloud = Cloud::<TelegramBackend>::new();
        
        let mut input_str;

        loop {
            input_str = String::new();
            std::io::stdin().read_line(&mut input_str).unwrap();

            let input_options = input_str.trim().split(' ').collect::<Vec<&str>>();

            match input_options[0] {
                "u" => {
                    cloud
                        .async_upload_file(
                            &PathBuf::from(input_options[1]),
                            Path::new(input_options[2])
                        )
                        .await
                        .unwrap();
                },
                "d" => {
                    cloud
                        .async_download_file(Path::new(input_options[1]))
                        .await
                        .unwrap();
                },
                _ => println!("Unsupported command")
            }

            input_str.clear();
        }
    });


        //let telegram_backend = TelegramBackend::new().await;

        // let f = VFSFile {
        //     name: "TestFile".to_string(),
        //     extension: "exe".to_string(),
        //     build_metafile: "build_file_PLvs8_Kv0hU.meta".to_string(),
        //     parts_name: vec![String::from("3b5929d3-a798-4a60-95d4-6ab40d072a79_1.part")],
        //     metadata: Default::default(),
        // };
        //telegram_backend.download_file(&f).await.expect("TODO: panic message");

        // let v_fs = VirtualFileSystem {
        //     dirs: HashMap::from([
        //         ("fs:".to_owned(),  Folder(VFSFolder {
        //             name: "Root".to_string(),
        //             metadata: Metadata,
        //             children: HashMap::from([
        //                 ("tttt".to_owned(),
        //                  File(VFSFile {
        //                      name: "tttt".to_string(),
        //                      extension: "xml".to_string(),
        //                      build_metafile: "11de4422-ead6-422c-b7aa-03b0e3fb8c65build_file_tttt.meta".to_string(),
        //                      parts_name: vec!["457f9f1f-ba15-4632-9f11-719ab588ca71_1.part".to_string()],
        //                      metadata: Metadata
        //                  }))
        //             ])
        //         })),
        //     ]),
        //     options: FSOption::default()
        // };


        //let cloud = Cloud::new(telegram_backend, None);
        //println!("Виртуальная файловая система: {}\n", cloud.get_fs_json());

        //println!("{:?}", cloud.get_file(Path::new("fs://tttt")));

        //cloud.async_upload_file(
        //    &PathBuf::from(r"C:\Users\nikiy\Documents\tttt.xml"),
        //    Path::new("fs://")
        //).await.unwrap();
        //println!("Файл выгружен");

        //println!("{}", cloud.get_fs_json());
        //cloud.get_fs_json();

        //telegram_backend.download_file(&vfile).await.unwrap();
        //cloud.async_download_file(Path::new("fs://tttt")).await.unwrap();
        //println!("Файл скачен");
}
