use std::path::{Path, PathBuf};
use crate::cloud::CloudError; //DELETE
use crate::virtual_file_system::VFSFile;

pub trait CloudBackend {
    fn sync_backend(&self) -> Result<(), CloudError>;
    fn upload_file(&self, file_path: &Path) -> Result<VFSFile, CloudError>;
    fn download_file(&self, file: &VFSFile) -> Result<PathBuf, CloudError>;
    fn check_file(&self, file_name: &str) -> bool;
    fn close(self) -> Result<(), CloudError>;
}

#[async_trait::async_trait]
pub trait AsyncCloudBackend {
    fn create() -> Self;
    async fn load_backend(&self) -> Result<(), CloudError>;
    async fn upload_file(&self, file_path: &Path) -> Result<(), CloudError>;
    async fn download_file(&self, file_path: &Path) -> Result<(), CloudError>;
    async fn remove_file(&self, file_path: &Path) -> Result<(), CloudError>;
    async fn check_file(&self, file_name: &str) -> bool;
    async fn close(self) -> Result<(), CloudError>;
}