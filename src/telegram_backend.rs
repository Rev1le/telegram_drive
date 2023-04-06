use std::{
    thread::JoinHandle,
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
    sync::{Arc, mpsc, Mutex,
           atomic::{AtomicBool, Ordering}
    }
};
use std::cell::RefCell;
use std::io::Write;
use std::sync::atomic::AtomicI64;
use std::sync::Barrier;

use serde_json::{json, Value};
use telegram_drive_core::{self, TDApp};

use crate::cloud::{CloudBackend, CloudError};
use crate::virtual_file_system::VFSFile;

type CallbackEvent = Box<dyn FnOnce(&TDApp) -> bool + Send + 'static>;

pub struct TelegramBackend {
    app_thread: JoinHandle<()>,
    sender: mpsc::Sender<CallbackEvent>,
    files_receive: mpsc::Receiver<Value>,
    files: RefCell<HashMap<String, i64>>
}

impl TelegramBackend {
    pub fn new() -> Self {

        TDApp::execute_query(&json!(
            {
                "@type": "setLogVerbosityLevel",
                "new_verbosity_level": 0
            }
        ).to_string()).unwrap();

        let (sender_event, receive_event) =
            mpsc::channel::<CallbackEvent>();
        let (sender_files, receive_files) =
            mpsc::channel::<Value>();

        let response_files = Arc::new(
            Mutex::new(HashMap::default())
        );
        let response_files_cl = Arc::clone(&response_files);

        let barrier = Arc::new(Barrier::new(2));
        let barrier_cln = Arc::clone(&barrier);

        let telegram_thread = std::thread::spawn(move || {

            let mut app = TDApp::new(
                response_files_cl,
                sender_files
            );
            app.account_auth();

            barrier_cln.wait();

            app.run(2.0, receive_event);
            println!("!!!=> Telegram has been closed <=!!!");
        });

        println!("!!!=> Wait <=!!!");
        barrier.wait();

        let dump_files = fs::read_to_string("TelegramBackend.json").unwrap_or(String::new());

        let files_hm = serde_json::from_str::<HashMap<String, i64>>(&dump_files).unwrap_or(HashMap::default());

        return Self {
            app_thread: telegram_thread,
            sender: sender_event,
            files: RefCell::new(files_hm),
            files_receive: receive_files
        }
    }
}

impl CloudBackend for TelegramBackend {

    fn sync_backend(&self) -> Result<(), CloudError> {
        todo!()
    }

    fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError> {

        println!("Upload file: {}", file_path.display());

        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();

        let barrier = Arc::new(Barrier::new(2));
        let barrier_cln = Arc::clone(&barrier);

        let file_id = Arc::new(AtomicI64::new(0));
        let file_id_cl = Arc::clone(&file_id);

        let callback = move |app: &TDApp| {

            let res_upload = app.upload_file(file_path);

            let file_id = res_upload.expect("Загрузка файла не удалась");
            file_id_cl.store(file_id, Ordering::SeqCst);

            barrier_cln.wait();

            true
        };

        self.sender.send(
            Box::new(callback)
        ).expect("Невозможно отправить callback в поток Telegram");

        barrier.wait();

        self.files.borrow_mut().insert(
            dbg!(file_name),
            dbg!(file_id.load(Ordering::SeqCst))
        );

        fs::write(
            "TelegramBackend.json",
            serde_json::to_string(&*self.files.borrow()).unwrap()
        ).unwrap();

        return Ok(());
    }

    fn download_file(&self, file: &VFSFile) -> Result<PathBuf, CloudError> {

        let files = self.files.borrow();

        let barrier = Arc::new(Barrier::new(2));

        for part in &file.parts_name {
            let part_id = *files.get(&*part).expect("Частица файла не найдена");

            let barrier_cln = Arc::clone(&barrier);

            let callback = move |app: &TDApp| {

                return match dbg!(app.download_file(part_id)) {
                    Ok(_) => {
                        barrier_cln.wait();
                        true
                    },
                    Err(_) => false
                }
            };

            self.sender.send(Box::new(callback)).unwrap();

            barrier.wait();

        }

        let build_file_id = *files.get( &file.build_metafile).expect("Частица файла не найдена");

        let execution_flag = Arc::new(AtomicBool::new(false));
        let execution_flag_cl = Arc::clone(&execution_flag);

        let callback = move |app: &TDApp| {

            let id = build_file_id;

            if let Ok(_) = dbg!(app.download_file(id)) {
                execution_flag_cl.store(true, Ordering::SeqCst);
                return true;
            }
            return false;
        };

        let metafile_path = format!("{}{}",r"F:\Projects\Rust\telegram_drive\td\file\documents\", &file.build_metafile);
        println!("Ожидание скачивание файла: {}", &metafile_path);

        self.sender.send(Box::new(callback)).unwrap();

        while execution_flag.load(Ordering::SeqCst) != true {
            continue;
        }

        println!("Загрузка файлов завершена");

        Ok(PathBuf::from(metafile_path))
    }

    fn check_file(&self, file_name: &str) -> bool {
        todo!();
    }

    fn close(self) -> Result<(), CloudError> {
        todo!()
    }
}
