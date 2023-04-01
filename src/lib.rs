pub mod virtual_file_system;
pub mod cloud;

use std::fs::File;
use std::path::PathBuf;
use serde_json::json;
use telegram_drive_core;
use telegram_drive_core::TDApp;
use crate::cloud::{Cloud, CloudError};

pub struct MockCloudBackend;

pub trait CloudBackend {
    fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError>;

    fn download_file(&self, file_name: &str) -> Result<PathBuf, CloudError>;

    fn check_file(&self, file_name: &str) -> bool;
}

impl CloudBackend for MockCloudBackend {
    fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError> {
        todo!()
    }

    fn download_file(&self, file_name: &str) -> Result<PathBuf, CloudError> {
        todo!()
    }

    fn check_file(&self, file_name: &str) -> bool {
        todo!()
    }
}

impl CloudBackend for TDApp {
    fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError> {
        self.send_query(&json!({
                    "@type": "sendMessage",
                    "chat_id": -1001976761155_i64,
                    "message_thread_id": 0,
                    "reply_to_message_id": 0,
                    "input_message_content": {
                        "@type": "inputMessageDocument",
                        "document": {
                            "@type": "inputFileLocal",
                            "path": file_path.display().to_string()
                        }
                    }
                }).to_string()).unwrap();
        Ok(())
    }

    fn download_file(&self, file_name: &str) -> Result<PathBuf, CloudError> {
        todo!()
    }

    fn check_file(&self, file_name: &str) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut cloud = Cloud::new(MockCloudBackend);
        print!("JSON VFS: {}", cloud.get_fs_json().unwrap());
    }
}
