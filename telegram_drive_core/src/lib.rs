use std::collections::HashMap;
use std::ffi::{c_char, c_double, c_int, c_void, CStr, CString};
use std::io::Write;
use serde_json::{Value, Map, json};
use std::sync::atomic::AtomicBool;
use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, mpsc, Mutex};

mod tdjson;
mod authentication;

use tdjson::*;

#[derive(Debug)]
pub struct TDApp {
    client_id: i32,
    are_authorized: bool,
    need_restart: bool,
    current_query_id: Cell<u64>,
    authentication_query_id: u64,
    users: HashMap<i64, Value>,
    chat_title: HashMap<i64, Value>,
    files: Arc<Mutex<HashMap<i64, Value>>>,
    file_sender: mpsc::Sender<serde_json::Value>
}

impl TDApp {
    pub fn new(response_files: Arc<Mutex<HashMap<i64, Value>>>, file_sender: mpsc::Sender<serde_json::Value>) -> Self {
        TDApp {
            client_id: unsafe {
                td_create_client_id()
            },
            are_authorized: false,
            need_restart: false,
            current_query_id: Cell::new(0),
            authentication_query_id: 0,
            users: HashMap::default(),
            chat_title: HashMap::default(),
            files: response_files,
            file_sender
        }
    }

    pub fn execute_query(request: &str) -> Result<Option<String>, std::ffi::NulError> {

        unsafe {
            let request = CString::new(request)?;

            let opt_response_str = td_execute(request.as_ptr())
                .as_ref()
                .map(
                    |chars|
                        CStr::from_ptr(chars).to_string_lossy().into_owned()
                );

            return Ok(opt_response_str)
        }
    }

    pub fn receive(&self, timeout: f64) -> Option<String> {
        unsafe {
            let response = td_receive(timeout);
            response.as_ref().map(
                |chars|
                    CStr::from_ptr(chars).to_string_lossy().into_owned()
            )
        }
    }

    pub fn send_query(&self, request: &str) -> Result<(), std::ffi::NulError> {
        unsafe {
            td_send(
                self.client_id,
                CString::new(request)?.as_ptr()
            );
        }

        self.current_query_id.set(
            self.current_query_id.get()+ 1
        );
        Ok(())
    }

    pub fn account_auth(&mut self) {
        println!("Авторизация...");
        std::io::stdout().flush().unwrap();

        authentication::set_tdlib_parameters(&self, None);

        loop {

            if let Some(json_str) = self.receive(2.0) {

                let json = serde_json::from_str::<Value>(&json_str)
                    .expect("Невалидный json");

                if json["@type"] == "error" {
                    self.error_handling(&json);
                }

                if json["@type"].as_str().unwrap() != "updateAuthorizationState" {

                    println!("Update ===> {}", json);
                    std::io::stdout().flush().unwrap();
                    continue
                }

                match json["authorization_state"]["@type"].as_str().unwrap() {

                    "authorizationStateWaitTdlibParameters" => authentication::set_tdlib_parameters(&self, None),
                    "authorizationStateWaitPhoneNumber" => authentication::set_phone_number(&self),
                    "authorizationStateWaitCode" => authentication::check_code(&self),
                    "authorizationStateWaitPassword" => authentication::check_password(&self),
                    "authorizationStateReady" => {
                        println!("|==|==|==> Authorization is completed <==|==|==|");
                        self.are_authorized = true;
                        return;
                    },
                    "authorizationStateClosed" => println!("Обновление статуса авторизации: {}", json),
                    "authorizationStateClosing" => println!("Обновление статуса авторизации: {}", json),
                    "authorizationStateLoggingOut" => {
                        self.are_authorized = false;
                        println!("|==|==|==> Logging out <==|==|==|")
                    },
                    "authorizationStateWaitEncryptionKey" => println!("Обновление статуса авторизации: {}", json),
                    "authorizationStateWaitOtherDeviceConfirmation" => println!("Обновление статуса авторизации: {}", json),
                    "authorizationStateWaitRegistration" => println!("Обновление статуса авторизации: {}", json),
                    _ => println!("Другие обновления авторизации: {}", json["authorization_state"]["@type"].as_str().unwrap())
                }
            }
        }
    }

    pub fn run(&mut self, timeout: f64, receive_thread: mpsc::Receiver<Box<dyn FnOnce(&TDApp) -> bool + Send + 'static>>) {
        println!("Tdlib run");

        self.send_query(&json!({
            "@type": "loadChats",
            "limit": 10,
        }).to_string()).unwrap();

        loop {

            if let Ok(callback) = receive_thread.try_recv() {
               callback(&self);
            }

            if let Some(str_json) = self.receive(timeout) {

                let json = serde_json::from_str::<Value>(&str_json)
                    .expect("Неудачный парсинг");

                self.parse_response(json);
                continue;
            }
            println!("None");
        }
    }

    fn parse_response(&self, json: Value) {

        match json["@type"].as_str().unwrap() {

            "error" => self.error_handling(&json),

            "message" => {
                //println!("==> Сообщение. {}", json);
            },

            "updateOption" => {
                let option_name = json["name"].as_str().unwrap();
                let option_value = &json["value"];
                //println!("==> Обновление опций. {} было обновлено на: {}\n", option_name, option_value)
            },

            "messages" => {
                //println!("==> Сообщения. {}\n", json);
            },

            "updateFile" => {
                let file_size = json["file"]["size"].as_f64().unwrap()/1024.0;
                let uploaded_size = json["file"]["remote"]["uploaded_size"].as_f64().unwrap()/1024.0;
                //println!("==> Загрузка файла. Размер файла: {:8} КБ, загружено: {:8} КБ\n", file_size.round(), uploaded_size.round());
            },

            "updateChatLastMessage" => {
                let chat_id = json["chat_id"].as_i64().unwrap();
                let last_message_content = &json["last_message"]["content"];
                //println!("==> Обновление чата. Чат: {} обновил последнее сообщение на: {}\n", chat_id, last_message_content);
            },

            "updateDeleteMessages" => {
                let chat_id = json["chat_id"].as_i64().unwrap();
                let delete_message_ids = &json["message_ids"];
                //println!("==> Обновление чата. В чате: {} удалены сообщения: {}\n", chat_id, delete_message_ids);
            },

            "updateMessageSendSucceeded" => {
                //println!("==> Обновление чата. {}", json);
            }
            _ => {}//println!("Обновление =>> {}\n", json),
        }
    }

    pub fn upload_file(&self, file_path: PathBuf) -> Result<i64, std::ffi::NulError> {

        let chat_id: i64 = -1001976761155;

        self.send_query(&json!({
                    "@type": "sendMessage",
                    "chat_id": chat_id,
                    "input_message_content": {
                        "@type": "inputMessageDocument",
                        "document": {
                            "@type": "inputFileLocal",
                            "path": file_path.display().to_string()
                        }
                    }
                }).to_string())?;


        // Возможны проблемы с парсингом json
        // Сделать проверку LocalFile.path == file_path

        println!("Запрос на отправку файла отправлен.");

        loop {
            if let Some(str_json) = self.receive(0.1) {
                let json = serde_json::from_str::<Value>(&str_json).expect("Неудачный парсинг");

                if json["@type"].as_str().unwrap() == "error" {
                    self.error_handling(&json);
                }

                if json["@type"].as_str().unwrap() == "updateFile" {
                    println!("\nTMP Обновление =>> {}", json);
                    std::io::stdout().flush().unwrap();
                }

                //println!("\nTMP Обновление =>> {}", json);
                //std::io::stdout().flush().unwrap();


                if json["@type"].as_str().unwrap() == "updateMessageSendSucceeded" {

                    println!("\nTMP Обновление =>> {}", json);
                    std::io::stdout().flush().unwrap();

                    let response_path = json["message"]
                        ["content"]
                        ["document"]
                        ["document"]
                        ["local"]
                        ["path"].as_str().unwrap();

                    if response_path != file_path.display().to_string().as_str() {
                        continue;
                    }

                    // Локальный id (Не равен remoteFile id)
                    let file_id = json["message"]["content"]["document"]["document"]["id"].as_i64().unwrap();

                    println!("Есть ли уже file_id в хеш таблице: {:?}", self.files.lock().unwrap().get(&file_id));
                    std::io::stdout().flush().unwrap();

                    self.files
                        .lock()
                        .unwrap()
                        .insert(
                            file_id,
                            json["message"]["content"]["document"].clone()
                        );

                    return Ok(file_id);
                }
            }
        }
    }

    pub fn download_file(&self, id: i64) -> Result<PathBuf, ()> {
        self.send_query(&json!({
            "@type": "downloadFile",
            "file_id": id,
            "priority": 1
        }).to_string()).unwrap();

        println!("Скачивание пошло....");

        loop {
            if let Some(str_json) = self.receive(0.1) {
                let json = serde_json::from_str::<Value>(&str_json).expect("Неудачный парсинг");

                println!("\nTMP_download Обновление =>> {}", json);
                std::io::stdout().flush().unwrap();

                if json["@type"].as_str().unwrap() == "updateFile" {
                    if json["file"]["local"]["is_downloading_completed"].as_bool().unwrap() {
                        return Ok(PathBuf::new())
                    }
                }
            }
        }
    }

    pub fn check_file(&self, file_name: &str) -> bool {
        todo!()
    }

    fn error_handling(&self, json: &Value) {
        println!("----------------------------------\nWARNING!!!!\n\n{}\n\n", json);
        fs::write(
            "test.txt",
            json.to_string().as_bytes()
        ).unwrap();
    }
}