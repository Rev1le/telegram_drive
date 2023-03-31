use std::cell::RefCell;
use std::path::{Path, PathBuf};
use crate::virtual_file_system::{VirtualFileSystem, FSOption, FileSystemNode, VFSError, VFSFile, VFSFolder};
use crate::MockCloudBackend;
use crate::virtual_file_system::FileSystemNode::File;

pub trait CloudBackend {
    fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError>;

    fn download_file(&self, file_name: &str) -> Result<PathBuf, CloudError>;

    fn check_file(&self, file_name: &str) -> bool;
}

#[derive(Debug)]
pub enum CloudError {
    IOError(std::io::Error)
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
                VirtualFileSystem::new(FSOption::default())
            ),
            backend,
        }
    }

    pub fn upload_file(&self, file_path: PathBuf) -> Result<(), CloudError> {
        Ok(())
    }

    pub fn get_fs_json(&self) -> String {
        String::default()
    }

    pub fn get_file(&self) {
        self.fs.borrow_mut().add_file(Path::new(r"fs://"), VFSFile {
            name: "first_file".to_string(),
            extension: "exe".to_string(),
            build_metafile: "".to_string(),
            metadata: Default::default(),
        }).unwrap();


        self.fs.borrow_mut().add_folder(Path::new(r"fs://"), VFSFolder {
            name: "test_folder".to_string(),
            metadata: Default::default(),
            children: Default::default(),
        }).unwrap();

        self.fs.borrow_mut().add_file(
            Path::new(r"fs://test_folder//"),
            VFSFile {
                name: "second_file".to_string(),
                extension: "exe".to_string(),
                build_metafile: "".to_string(),
                metadata: Default::default(),
            }
        ).unwrap();

        self.fs.borrow_mut().add_file(
            Path::new(r"fs://test_folder//"),
            VFSFile {
                name: "third_file".to_string(),
                extension: "exe".to_string(),
                build_metafile: "".to_string(),
                metadata: Default::default(),
            }
        ).unwrap();

        println!("Dddd");

        self.fs.borrow().print_fs();

        println!("Получение файла DEBUG {:#?}", self.fs.borrow_mut().get_mut_file(Path::new(r"fs://test")));

        println!("{}", self.fs.borrow().get_fs_as_json().unwrap());
    }

    pub fn download_file(&self, file_name: &str) -> Result<PathBuf, CloudError> {
        Ok(PathBuf::new())
    }
}