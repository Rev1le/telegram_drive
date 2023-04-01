use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use crate::virtual_file_system::{VirtualFileSystem, FSOption, FileSystemNode, VFSError, VFSFile, VFSFolder};
use crate::{CloudBackend, MockCloudBackend};
use crate::virtual_file_system::FileSystemNode::File;

use telegram_drive_file::*;

#[derive(Debug)]
pub enum CloudError {
    IOError(std::io::Error),
    Test
}

impl From<VFSError> for CloudError {
    fn from(value: VFSError) -> Self {
        return match value {
            VFSError::Test => CloudError::Test,
            _ => {panic!("Unsupported errors")}
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cloud<T: CloudBackend> {
    fs: RefCell<VirtualFileSystem>,
    backend: T
}

impl<T: CloudBackend> Cloud<T> {

    pub fn new(backend: T) -> Self {
        Cloud {
            fs: RefCell::new(
                VirtualFileSystem::new(
                    FSOption::default()
                )
            ),
            backend,
        }
    }

    pub fn get_fs_json(&self) -> String {
        self.fs.borrow().display()
    }

    pub fn upload_file(&self, file_path: &Path, virtial_path: &Path) -> Result<(), CloudError> {

        let file_path = PathBuf::from(file_path);

        let path_for_save_parts = PathBuf::from("W:\\tmp_tel_drive\\");

        let options_encode = Options {
            path_for_save: Some(path_for_save_parts.clone()),
            count_parts: None,
            part_size: None,
            compressed: None,
        };

        let _ = file_separation::encode_file(&file_path, options_encode).unwrap();

        for entry in fs::read_dir(path_for_save_parts).unwrap() {
            let entry_path = entry.as_ref().unwrap().path();

            println!("Найден файл: {:?}", &entry_path);
            self.backend.upload_file(entry_path.clone()).unwrap();
        }

        let _ = self.fs.borrow_mut().add_file(virtial_path, VFSFile {
            name: "".to_string(),
            extension: "".to_string(),
            build_metafile: "".to_string(),
            parts_name: vec![],
            metadata: Default::default(),
        });

        Ok(())
    }

    pub fn download_file(&self, file_path: &Path) -> Result<PathBuf, CloudError> {

        let binding = self.fs.borrow();
        let file = binding.get_file(file_path)?;

        for part in &file.parts_name {
            let _ = dbg!(self.backend.download_file(part)?);
        }

        let metafile = dbg!(self.backend.download_file(&file.build_metafile)?);

        // объединяем файл

        Ok(PathBuf::new())
    }

    pub fn get_file(&self, path: &Path) -> Result<VFSFile, CloudError> {
        self.fs
            .borrow()
            .get_file(path)
            .map(|file| file.clone())
            .map_err(|err|err.into())
    }

    pub fn get_folder(&self, path: &Path) -> Result<VFSFolder, CloudError> {
        self.fs
            .borrow()
            .get_folder(path)
            .map(|folder| folder.clone())
            .map_err(|err|err.into())
    }
}

// META файл именуется одинаково (при загрузке одинаковых файлов идет перезапись meta файла)