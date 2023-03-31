pub mod virtual_file_system;
pub mod cloud;

use std::fs::File;
use std::path::PathBuf;
use telegram_drive_core;
use crate::cloud::{Cloud, CloudBackend, CloudError};

pub struct MockCloudBackend;

impl cloud::CloudBackend for MockCloudBackend {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let c = Cloud::new(MockCloudBackend);
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
