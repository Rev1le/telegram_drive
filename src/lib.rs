pub mod virtual_file_system;
pub mod cloud;

use std::{
    thread::JoinHandle,
    collections::HashMap,
    fs::File,
    path::PathBuf,
    sync::{Arc, mpsc, Mutex,
        atomic::{AtomicBool, Ordering}
    }
};

use serde_json::{json, Value};
use telegram_drive_core::{self, TDApp};
use crate::cloud::{Cloud, CloudError};

pub struct TelegramBackend {
    app_thread: JoinHandle<()>,
    sender: mpsc::Sender<Box<dyn FnOnce(&TDApp) -> bool + Send + 'static>>,
    files: Arc<Mutex<HashMap<i64, Value>>>,
    files_receive: std::sync::mpsc::Receiver<serde_json::Value>
}

impl TelegramBackend {
    pub fn new() -> Self {
        let (sender_event, receive_event) = mpsc::channel::<Box<dyn FnOnce(&TDApp) -> bool + Send + 'static>>();

        let (sender_files, receive_files) = mpsc::channel::<serde_json::Value>();

        let response_files = Arc::new(Mutex::new(HashMap::default()));
        let response_files_cl = Arc::clone(&response_files);

        let execution_flag = Arc::new(AtomicBool::new(false));
        let execution_flag_cl = Arc::clone(&execution_flag);

        let telegram_thread = std::thread::spawn(move || {
            let mut app = TDApp::new(response_files_cl, sender_files);
            let execution_flag = execution_flag_cl;

            app.account_auth();

            execution_flag.store(true, Ordering::Relaxed);
            app.run(0.5, receive_event);

            println!("!!!=> Telegram has been closed <=!!!");
        });

        loop {
            if execution_flag.load(Ordering::Relaxed) == true {
                return Self {
                    app_thread: telegram_thread,
                    sender: sender_event,
                    files: response_files,
                    files_receive: receive_files
                }
            }
        }
    }
}

impl CloudBackend for TelegramBackend {

    fn get_remote_ids(&self) -> Vec<i64> {

        let lock_files = self.files.lock().unwrap();
        let mut output_ids = Vec::with_capacity(lock_files.len());

        lock_files
            .keys()
            .map(|id| output_ids.push(*id))
            .for_each(drop);

        return output_ids;

        //println!("{}", serde_json::to_string(&*self.files.lock().unwrap()).unwrap());
    }

    fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError> {

        let execution_flag = Arc::new(AtomicBool::new(false));
        let execution_flag_cl = Arc::clone(&execution_flag);

        let callback = |app: &TDApp| {
            println!("Загрузка файла: {}", file_path.display());

            let execution_flag = execution_flag_cl;

            if let Ok(_) = app.upload_file(file_path) {
                execution_flag.store(true, Ordering::SeqCst);
                return true;
            }
            panic!("Загрузка файла не удалась");
            return false;
        };

        match self.sender.send(Box::new(callback)) {
            Ok(_) => {}
            Err(_) => panic!("Невозможно отправить callback в поток Telegram")
        }

        loop {
            if execution_flag.load(Ordering::SeqCst) != true {
                continue
            } else {
                return Ok(());
            }
        }
    }

    fn download_file(&self, file_name: &str, id: i64) -> Result<PathBuf, CloudError> {

        let file_name = file_name.to_owned();

        let callback = move |app: &TDApp| {

            if let Ok(_) = app.download_file(id) {
                return true;
            }
            return false;
        };

        self.sender.send(Box::new(callback)).unwrap();
        Ok(PathBuf::new())
    }

    fn check_file(&self, file_name: &str) -> bool {

        todo!();

        let file_name = file_name.to_owned();

        let callback = |app: &TDApp| {
            let file_name = file_name;

            app.check_file(&file_name)
        };

        self.sender.send(Box::new(callback)).unwrap();
        return true;
    }
}
