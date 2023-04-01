use std::collections::HashMap;
use std::ffi::{c_char, c_double, c_int, c_void, CStr, CString};
use std::io::Write;
use serde_json::{Value, Map, json};
use std::sync::atomic::AtomicBool;
use std::cell::Cell;

mod tdjson;
mod authentication;

use tdjson::*;

#[derive(Debug, Clone)]
pub struct TDApp {
    client_id: i32,
    authorization_state: Value,
    are_authorized: bool,
    need_restart: bool,
    current_query_id: u64,
    authentication_query_id: u64,
    users: HashMap<i64, Value>,
    chat_title: HashMap<i64, Value>,
}

impl TDApp {
    pub fn new() -> Self {
        TDApp {
            client_id: unsafe {
                td_create_client_id()
            },
            authorization_state: Value::Null,
            are_authorized: false,
            need_restart: false,
            current_query_id: 0,
            authentication_query_id: 0,
            users: HashMap::default(),
            chat_title: HashMap::default(),
        }
    }

    pub fn execute_query(request: &str) -> Result<Option<String>, std::ffi::NulError> {
        unsafe {
            let response = td_execute(CString::new(request)?.as_ptr());
            let response_str = response.as_ref().map(
                |chars|
                    CStr::from_ptr(chars)
                        .to_string_lossy()
                        .into_owned()
            );
            return Ok(response_str)
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
        Ok(())
    }

    pub fn account_auth(&mut self) {
        print!("вызов пошел");
        authentication::set_tdlib_parameters(&self, None);
        loop {
            if let Some(json_str) = self.receive(5.0) {
                let json = serde_json::from_str::<Value>(&json_str).expect("Невалидный json");

                if json["@type"].as_str().unwrap() != "updateAuthorizationState" {
                    println!("Update ===> {}", json);
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
}
