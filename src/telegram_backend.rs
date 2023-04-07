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
use async_trait::async_trait;

use serde_json::{json, Value};
use telegram_drive_core::{self, TDApp};

use crate::cloud::{AsyncCloudBackend, CloudBackend, CloudError};
use crate::virtual_file_system::VFSFile;

const CLOUD_CHAT_ID: i64 = -1001976761155;

type CallbackEvent = Box<dyn FnOnce(&TDApp) -> bool + Send + 'static>;

#[derive(Debug)]
pub struct TelegramBackend {
    telegram: TDApp,
    cloud_chat: Value,
    files: HashMap<String, (i64, Value)>,
}

impl TelegramBackend {
    pub async fn new() -> Self {

        // Отключение вывод логов в консоль
        TDApp::execute_query(&json!({
            "@type": "setLogVerbosityLevel",
            "new_verbosity_level": 0
        }).to_string()).await.unwrap();

        // Создание телеграм клиента
        let mut app = TDApp::create().await;
        app.account_auth().await.expect("Ошибка авторизации в Telegram");
        app.skip_all_update(0.1);

        let cloud_chat = app.get_chat(CLOUD_CHAT_ID).await;

        let messages = app.load_messages(CLOUD_CHAT_ID).await;

        //dbg!(app.get_message(messages[5], CLOUD_CHAT_ID).await);
        //println!("{}", serde_json::to_string(&messages[0..5]).unwrap());

        return Self {
            telegram: app,
            cloud_chat,
            files: TelegramBackend::get_files_from_message(messages),
        }
    }

    fn get_files_from_message(messages: Vec<Value>) -> HashMap<String, (i64, Value)> {

        let mut files_hm = HashMap::with_capacity(messages.len());

        for message in messages {
            let message_id = message["id"].as_i64().unwrap();

            if !message["content"]["document"].is_null() {

                let document = &message["content"]["document"];

                let file_name = document["file_name"].as_str().unwrap().to_owned();
                let file_id = document["document"]["id"].as_i64().unwrap();

                files_hm.insert(file_name.to_owned(), (file_id, message));
            }
        }

        files_hm
    }
}

#[async_trait::async_trait]
impl AsyncCloudBackend for TelegramBackend {
    async fn load_backend(&self) -> Result<(), CloudError> {
        // let res_send_query = self.telegram.borrow_mut().send_query(&json!({
        //     "@type": "ddd",
        //     "chat_id": CLOUD_CHAT_ID
        // }).to_string());

        Ok(())
    }

    async fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError> {
        todo!()
    }

    async fn download_file(&self, file: &VFSFile) -> Result<PathBuf, CloudError> {
        let file_parts = &file.parts_name;
        let build_file = &file.build_metafile;

        for part_name in file_parts {
            if let Some((file_id, _)) = self.files.get(part_name) {

                if let Err(_) = File::open(format!("td/file/documents/{}", part_name)) {
                    self.telegram.download_file(*file_id).await;
                } else {
                    println!("Файл уже скачен.");
                }
            }
        }

        if let Some((file_id, _)) = self.files.get(build_file) {

            if let Err(_) = File::open(format!("td/file/documents/{}", build_file)) {
                self.telegram.download_file(*file_id).await;
            } else {
                println!("МетаФайл уже скачен.");
            }

        }

        Ok(PathBuf::new())
    }

    async fn check_file(&self, file_name: &str) -> bool {
        todo!()
    }

    async fn close(self) -> Result<(), CloudError> {
        todo!()
    }
}
