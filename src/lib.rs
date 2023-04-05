use std::{thread::JoinHandle, collections::HashMap, fs::File, path::PathBuf, sync::{Arc, mpsc, Mutex,
                                                                                    atomic::{AtomicBool, Ordering}
}, fs};
use std::cell::RefCell;
use std::sync::atomic::AtomicI64;

pub mod virtual_file_system;
pub mod cloud;

use serde_json::{json, Value};
use telegram_drive_core::{self, TDApp};
use crate::cloud::{Cloud, CloudError, CloudBackend};
use crate::virtual_file_system::VFSFile;

type CallbackEvent = Box<dyn FnOnce(&TDApp) -> bool + Send + 'static>;

pub struct TelegramBackend {
    app_thread: JoinHandle<()>,
    sender: mpsc::Sender<CallbackEvent>,
    files_receive: mpsc::Receiver<Value>,
    files: RefCell<HashMap<String, i64>>,
}

impl TelegramBackend {
    pub fn new() -> Self {

        let (sender_event, receive_event) =
            mpsc::channel::<CallbackEvent>();
        let (sender_files, receive_files) =
            mpsc::channel::<Value>();

        let response_files = Arc::new(
            Mutex::new(HashMap::default())
        );
        let response_files_cl = Arc::clone(&response_files);

        let execution_flag = Arc::new(AtomicBool::new(false));
        let execution_flag_cl = Arc::clone(&execution_flag);

        let telegram_thread = std::thread::spawn(move || {

            let mut app = TDApp::new(
                response_files_cl,
                sender_files
            );
            app.account_auth();

            execution_flag_cl.store(true, Ordering::Relaxed);

            app.run(2.0, receive_event);
            println!("!!!=> Telegram has been closed <=!!!");
        });

        loop {
            if execution_flag.load(Ordering::Relaxed) == true {
                return Self {
                    app_thread: telegram_thread,
                    sender: sender_event,
                    files: RefCell::new(HashMap::default()),
                    files_receive: receive_files
                }
            }
        }
    }
}

impl CloudBackend for TelegramBackend {

    fn sync_backend(&self) -> Result<(), CloudError> {
        todo!()
    }

    fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError> {

        println!("Upload file: {}", file_path.display());

        let execution_flag = Arc::new(AtomicBool::new(false));
        let execution_flag_cl = Arc::clone(&execution_flag);

        let file_id = Arc::new(AtomicI64::new(0));
        let file_id_cl = Arc::clone(&file_id);

        let file_path_cl = file_path.clone();

        let callback = move |app: &TDApp| {

            let res_upload = app.upload_file(file_path_cl);

            if let Ok(file_id) = res_upload {
                execution_flag_cl.store(true, Ordering::SeqCst);
                file_id_cl.store(file_id, Ordering::SeqCst);
                return true;
            }

            panic!("Загрузка файла не удалась");
        };

        if let Err(e) = self.sender.send(Box::new(callback)) {
            panic!("Невозможно отправить callback в поток Telegram с ошибкой: {:?}", e);
        }

        while execution_flag.load(Ordering::SeqCst) != true { continue; }

        self.files.borrow_mut().insert(
            dbg!(file_path.file_name().unwrap().to_string_lossy().to_string()),
            dbg!(file_id.load(Ordering::SeqCst))
        );

        return Ok(());
    }

    fn download_file(&self, file: &VFSFile) -> Result<PathBuf, CloudError> {

        let files = self.files.borrow();

        for part in &file.parts_name {
            let part_id = *files.get(&*part).expect("Частица файла не найдена");

            let execution_flag = Arc::new(AtomicBool::new(false));
            let execution_flag_cl = Arc::clone(&execution_flag);

            let callback = move |app: &TDApp| {

                let id = part_id;

                if let Ok(_) = dbg!(app.download_file(id)) {
                    execution_flag_cl.store(true, Ordering::SeqCst);
                    return true;
                }
                return false;
            };

            self.sender.send(Box::new(callback)).unwrap();

            while execution_flag.load(Ordering::SeqCst) != true {
                continue;
            }

        }

        let build_file_id =*files.get( &file.build_metafile).expect("Частица файла не найдена");

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
}
