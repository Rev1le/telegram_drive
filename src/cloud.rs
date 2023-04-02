use std::cell::RefCell;
use std::{fs, thread};
use std::path::{Path, PathBuf};
use std::time::Duration;
use crate::virtual_file_system::{VirtualFileSystem, FSOption, FileSystemNode, VFSError, VFSFile, VFSFolder};
use crate::{CloudBackend};
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

pub trait CloudBackend {
    fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError>;

    fn download_file(&self, file_name: &str, id: i64) -> Result<PathBuf, CloudError>;

    fn check_file(&self, file_name: &str) -> bool;

    fn get_remote_ids(&self) -> Vec<i64>;
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

    pub fn upload_file(&self, file_path: &Path, virtual_path: &Path) -> Result<(), CloudError> {

        let file_path = PathBuf::from(file_path);

        let path_for_save_parts = PathBuf::from(r"W:\tmp_tel_drive\");

        let options_encode = Options {
            path_for_save: Some(path_for_save_parts.clone()),
            count_parts: None,
            part_size: None,
            compressed: None,
        };

        let separation_file = file_separation::encode_file(&file_path, options_encode).unwrap();

        let mut parts_name = vec![];

        for part in &separation_file.parts {
            println!("Найден файл: {:?}", part);
            parts_name.push(part.part_file_name.clone());
            let mut part_path = path_for_save_parts.clone();

            part_path.push(part.part_file_name.clone());

            self.backend.upload_file(part_path).unwrap();
        }

        let mut metafile_path = path_for_save_parts.clone();
        metafile_path.push(&separation_file.metafile);

        self.backend.upload_file(metafile_path).unwrap();

        let _ = self.fs.borrow_mut().add_file(virtual_path, VFSFile {
            name: String::from("testfile"),
            extension: separation_file.file_extension.clone(),
            remote_ids: dbg!(self.backend.get_remote_ids()),
            build_metafile: separation_file.metafile.clone(),
            parts_name,
            metadata: Default::default(),
        }).unwrap();

        println!("Выгрузка файла завершена");

        Ok(())
    }

    pub fn download_file(&self, file_path: &Path) -> Result<PathBuf, CloudError> {

        let binding = self.fs.borrow();

        fs::write("docs.json", serde_json::to_string(&*binding).unwrap().as_bytes()).unwrap();

        let file = dbg!(binding.get_file(file_path))?;

        for id in &file.remote_ids {
            let _ = dbg!(self.backend.download_file("part", *id));
        }

        //let metafile = dbg!(self.backend.download_file(&file.build_metafile)?);

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

/*
let callback = |app: &TDApp| {
            app.check_file("d");
            if let Ok(_) = app.upload_file(PathBuf::new()) {
                true
            } else {
                false
            }
        };

        sender.send(Box::new(callback)).unwrap();
 */