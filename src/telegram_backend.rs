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

pub struct TelegramBackend {
    telegram: TDApp,
    cloud_chat: Value
}

impl TelegramBackend {
    pub async fn new() -> Self {

        // Отключение вывод логов в консоль
        TDApp::execute_query(&json!({
            "@type": "setLogVerbosityLevel",
            "new_verbosity_level": 0
        }).to_string()).await.unwrap();

        // Создание телегра клиента
        let mut app = TDApp::create().await;
        app.account_auth().await.expect("Ошибка авторизации в Telegram");

        //let dump_files = fs::read_to_string("TelegramBackend.json").unwrap_or(String::new());

        //app.get_chat(CLOUD_CHAT_ID).await;
        let messages = app.load_messages(CLOUD_CHAT_ID).await;

        //dbg!(app.get_message(messages[5], CLOUD_CHAT_ID).await);

        println!("{}", serde_json::to_string(&messages[0..5]).unwrap());


        loop {

        }

        return Self {
            telegram: app,
            cloud_chat: Value::default()
        }
    }
}

#[async_trait::async_trait]
impl AsyncCloudBackend for TelegramBackend {
    async fn sync_backend(&self) -> Result<(), CloudError> {
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
        todo!()
    }

    async fn check_file(&self, file_name: &str) -> bool {
        todo!()
    }

    async fn close(self) -> Result<(), CloudError> {
        todo!()
    }
}
