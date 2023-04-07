#![feature(cell_update)]
#![feature(async_iterator)]

use std::collections::HashMap;
use std::ffi::{c_char, c_double, c_int, c_void, CStr, CString};
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::cell::{Cell, RefCell};
use std::{ffi, fs, io};
use std::async_iter::AsyncIterator;
use std::fmt::format;
use std::fs::File;
use std::path::{Iter, Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, mpsc, Mutex};
use std::task::{Context, Poll};
use std::time::SystemTime;

use serde_json::{Value, json};

use tokio::{
    io::{
        self as async_io,
        AsyncWriteExt
    },
    fs::{
        self as async_fs,
        File as AsyncFile
    },
    sync::RwLock as AsyncRwLock
};

mod tdjson;
mod authentication;
pub mod error;

use authentication as auth;

use tdjson::*;
use error::*;


#[derive(Debug)]
pub struct TDApp {
    client_id: i32,
    are_authorized: bool,
    current_query_id: u64,
    error_log_file: AsyncFile,
}

impl TDApp {

    pub async fn create() -> Self {

        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        fs::create_dir("logs");

        TDApp {
            client_id: unsafe { td_create_client_id() },
            are_authorized: false,
            current_query_id: 0,
            error_log_file: AsyncFile::create(format!("logs/error_{}.log", time)).await.unwrap()
        }
    }

    pub async fn execute_query(request: &str) -> Result<Option<String>, std::ffi::NulError> {

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

    pub async fn receive(&mut self, timeout: f64) -> Option<String> {
        self.sync_receive(timeout)
    }

    pub fn sync_receive(&mut self, timeout: f64) -> Option<String> {
        unsafe {
            let response = td_receive(timeout);

            return response.as_ref().map(
                |chars|
                    CStr::from_ptr(chars).to_string_lossy().into_owned()
            )

        }
    }

    pub async fn send_query(&mut self, request: &str) -> Result<(), std::ffi::NulError> {
        unsafe {
            td_send(
                self.client_id,
                CString::new(request)?.as_ptr()
            );
        }

        self.current_query_id += 1;

        Ok(())
    }

    pub async fn account_auth(&mut self) -> Result<(), TDAppError> {

        println!("Авторизация...");
        async_io::stdout().flush().await?;

        TDApp::execute_query(&json!({
            "@type": "setOption",
            "name": "ignore_background_updates",
            "value": "true"
        }).to_string()).await.expect("Строка содержала нулебой байт");

        self.send_query(
            &authentication::get_tdlib_params_request(None)
        ).await.expect("Строка содержала нулебой байт");

        while !self.are_authorized {
            if let Some(response) = self.receive(1.0).await {

                let json = serde_json::from_str::<Value>(&response)
                    .expect("TDLib прислал невалидный json");

                if json["@type"] == "error" {
                    self.error_handling(&json).await?;
                }


                let json_type = json["@type"]
                    .as_str()
                    .expect("TDLib прислал невалидный json");

                if json_type != "updateAuthorizationState" {

                    async_io::stdout().write_all(format!("Update ===> {}\n----\n", json).as_bytes()).await?;
                    async_io::stdout().flush().await?;
                    continue

                }
                println!("AuthUpdate ===> {}\n----\n", json);
                async_io::stdout().flush().await?;

                let authorization_state = json["authorization_state"]["@type"]
                    .as_str()
                    .expect("TDLib прислал невалидный json");

                match authorization_state {

                    "authorizationStateWaitTdlibParameters" => {
                        println!("Sending TdlibParameters");
                        async_io::stdout().flush().await?;

                        self.send_query(
                            &auth::get_tdlib_params_request(None)
                        ).await.expect("Строка содержала нулебой байт");

                    },

                    "authorizationStateWaitPhoneNumber" =>
                        self.send_query(&auth::get_phone_number_request()).await.unwrap(),

                    "authorizationStateWaitCode" =>
                        self.send_query(&auth::get_check_code_request()).await.unwrap(),

                    "authorizationStateWaitPassword" =>
                        self.send_query(&auth::get_check_password_request()).await.unwrap(),

                    "authorizationStateReady" => {
                        println!("|==|==|==> Authorization is completed <==|==|==|");

                        self.are_authorized = true;
                        continue;
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
                    _ => println!("Другие обновления авторизации: {}", authorization_state)
                }
            }
        }

        Ok(())
    }

    async fn error_handling(&mut self, json: &Value) -> io::Result<()> {

        println!("----------------------------------\nFOUND ERROR: {}\n", json);

        let error_message = json["message"].as_str().expect("TDLib прислал невалидный json");

        match error_message {
            "PHONE_NUMBER_INVALID" =>
                self.send_query(&auth::get_phone_number_request()).await.unwrap(),
            "PHONE_CODE_INVALID" =>
                self.send_query(&auth::get_check_code_request()).await.unwrap(),
            "PASSWORD_HASH_INVALID" =>
                self.send_query(&auth::get_check_password_request()).await.unwrap(),
            _ => println!("Unsupported telegram error")
        }

        return self.error_log_file
            .write_all(
                format!("\nError ==> {json}\n|").as_bytes()
            ).await

    }

    pub async fn get_chat(&mut self, chat_id: i64) -> Value {
        self.send_query(&json!({
            "@type": "getChat",
            "chat_id": chat_id
        }).to_string()).await.unwrap();

        // self.send_query(&json!({
        //     "@type": "loadChats",
        //     "limit": 10
        // }).to_string()).await.unwrap();

        loop {
            if let Some(response) = self.receive(1.0).await {

                let json = serde_json::from_str::<Value>(&response)
                    .expect("TDLib прислал невалидный json");

                if json["@type"] == "error" {
                    self.error_handling(&json).await.unwrap();
                }

                if json["@type"] == "chat" {
                    println!("chat: {}\n", json);
                    panic!("CONTROLL")
                } else {
                    println!("Update: {}\n", json);
                }
            }
        }
    }

    pub async fn load_messages(&mut self, chat_id: i64) -> Vec<Value> {
        self.send_query(&json!({
            "@type": "getChatHistory",
            "chat_id": chat_id,
            "limit": 20
        }).to_string()).await.unwrap();

        let mut chat_all_messages = vec![];

        while let Some(json_update) = self.next() {

            if json_update["@type"] == "messages" {

                let total_count = json_update["total_count"].as_u64().unwrap();

                if total_count <= 0 {
                    println!("Сообщений больще нет");

                    return chat_all_messages
                }

                let messages = json_update["messages"].as_array().unwrap();
                chat_all_messages.extend_from_slice(&messages);

                println!("messages: {}\n", json_update);

                let last_message_id =
                    if let Some(message) = messages.last() {
                        message["id"].as_i64().unwrap()
                    } else {
                        return chat_all_messages
                    };

                self.send_query(&json!({
                        "@type": "getChatHistory",
                        "chat_id": chat_id,
                        "limit": 30,
                        "from_message_id": last_message_id
                    }).to_string()).await.unwrap();

            } else {
                //println!("Update: {}\n", json);
            }
        }

        vec![]

        /*
        loop {
            if let Some(response) = self.receive(1.0).await {

                let json = serde_json::from_str::<Value>(&response)
                    .expect("TDLib прислал невалидный json");

                if json["@type"] == "error" {
                    self.error_handling(&json).await.unwrap();
                }

                if json["@type"] == "messages" {

                    let total_count = json["total_count"].as_u64().unwrap();

                    if total_count <= 0 {
                        println!("Сообщений больще нет");

                        return chat_all_messages
                    }

                    let messages = json["messages"].as_array().unwrap();
                    chat_all_messages.extend_from_slice(&messages);

                    println!("messages: {}\n", json);

                    let last_message_id = messages.last().unwrap()["id"].as_i64().unwrap();

                    self.send_query(&json!({
                        "@type": "getChatHistory",
                        "chat_id": chat_id,
                        "limit": 30,
                        "from_message_id": last_message_id
                    }).to_string()).await.unwrap();

                } else {
                    //println!("Update: {}\n", json);
                }
            }
        }

         */
    }

    pub async fn get_message(&mut self, message_id: i64, chat_id: i64) -> Value {

        self.send_query(&json!({
            "@type": "getMessage",
            "chat_id": chat_id,
            "message_id": message_id
        }).to_string()).await.unwrap();

        loop {
            if let Some(response) = self.receive(1.0).await {

                let json = serde_json::from_str::<Value>(&response)
                    .expect("TDLib прислал невалидный json");

                if json["@type"] == "error" {
                    self.error_handling(&json).await.unwrap();
                }

                if json["@type"] == "message" {

                    if json["chat_id"] == chat_id && json["id"] == message_id {
                        return json
                    }
                }
            }
        }
    }
}

impl Iterator for TDApp {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(response) = self.sync_receive(1.0) {

            let json = serde_json::from_str::<Value>(&response)
                .expect("TDLib прислал невалидный json");

            if json["@type"] == "error" {
                futures::executor::block_on(self.error_handling(&json)).unwrap();
            }

            return Some(json)
        }

        None
    }
}

// impl IntoIterator for &mut TDApp {
//     type Item = Value;
//     type IntoIter = Self;
//
//     fn into_iter(self) -> Self::IntoIter {
//         self
//     }
// }
