use std::cell::RefCell;
use std::{fs, io, thread};
use std::path::{Path, PathBuf};
use std::time::Duration;
use crate::virtual_file_system::{VirtualFileSystem, FSOption, FileSystemNode, VFSError, VFSFile, VFSFolder};
use std::fs::File;
use std::io::{ErrorKind, Read, Write};

use telegram_drive_file::{Options as SeparationOptions, *};
use telegram_drive_file::file_separation::{EncodeErrors, SeparationFile};
use crate::cloud_backend::{AsyncCloudBackend, CloudBackend};

#[derive(Debug)]
pub enum CloudError {
    IOError(std::io::Error),
    EncodeError(EncodeErrors),
    VFSError(VFSError),
}

impl From<VFSError> for CloudError {
    fn from(value: VFSError) -> Self {
        Self::VFSError(value)
    }
}
impl From<EncodeErrors> for CloudError {
    fn from(value: EncodeErrors) -> Self {
        Self::EncodeError(value)
    }
}

#[derive(Debug, Clone)]
struct CloudOptions {
    work_dir: PathBuf
}

#[derive(Debug, Clone)]
pub struct Cloud<T: AsyncCloudBackend> {
    fs: RefCell<VirtualFileSystem>,
    backend: T,
    option: CloudOptions,
}

impl<T: AsyncCloudBackend> Cloud<T> {

    pub fn new() -> Self {

        let try_open_vfs = File::open("vfs.json");

        let vfs_from_backup =
            match try_open_vfs {
                Ok(mut f) => serde_json::from_reader::<File, VirtualFileSystem>(f).unwrap(),

                Err(e) => match e.kind() {
                    ErrorKind::NotFound => VirtualFileSystem::new(FSOption::default()),
                    _ => panic!("{}", e)
                }
            };

        Cloud {
            fs: RefCell::new(vfs_from_backup),
            backend: T::create(),
            option: CloudOptions {
                work_dir: PathBuf::from("./td/file/documents/")
            },
        }
    }

    fn save_vfs(&self) -> io::Result<()> {
        // let result_f = dbg!(File::open("vfs.json"));
        // match result_f {
        //     Ok(mut f) => {
        //         f.write_all(&vec![]).unwrap();
        //     }
        //     Err(e) => match e.kind() {
        //         ErrorKind::NotFound => {
        //             let vfs_json = serde_json::to_string(&*self.fs.borrow()).unwrap();
        //             fs::write("vfs.json", vfs_json.as_bytes())?;
        //         }
        //         _ => return Err(e)
        //     }
        // }
        let vfs_json = serde_json::to_string(&*self.fs.borrow()).unwrap();
        fs::write("vfs.json", vfs_json.as_bytes())?;

        Ok(())
    }

    pub fn get_fs_json(&self) -> String {
        serde_json::to_string(&*self.fs.borrow()).unwrap()
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

    pub async fn async_upload_file(
        &self,
        file_path: &PathBuf,
        virtual_path: &Path
    ) -> Result<(), CloudError> {
        use telegram_drive_file::file_separation;

        let options = SeparationOptions {
            path_for_save: Some(self.option.work_dir.clone()),
            count_parts: None,
            part_size: None,
            compressed: None,
        };

        let separation_file =
            dbg!(file_separation::encode_file(dbg!(file_path), options)?);

        self.add_file_to_vfs(&separation_file, virtual_path)?;

        for part_file in &separation_file.parts {

            let mut part_path = self.option.work_dir.clone();
            part_path.push(&part_file.part_file_name);

            self.backend.upload_file(&part_path).await?;
        }

        let mut metafile_path = self.option.work_dir.clone();
        metafile_path.push(&separation_file.metafile);

        self.backend.upload_file(&metafile_path).await?;

        self.save_vfs().unwrap();
        return Ok(());
    }

    pub async fn async_download_file(&self, virtual_path: &Path) -> Result<PathBuf, CloudError> {
        use telegram_drive_file::file_assembly;

        let v_fs = self.fs.borrow();
        let v_file = v_fs.get_file(virtual_path)?;

        for part in &v_file.parts_name {

            let part_path = format!("{}{}", self.option.work_dir.display(), part);
            let _ = self.backend.download_file(Path::new(&part_path)).await.unwrap();
        }
        let _ = self.backend.download_file(Path::new(&v_file.build_metafile)).await.unwrap();

        let metafile_path = format!("{}{}", self.option.work_dir.display(), v_file.build_metafile);

        //let metafile_path = format!("{}{}", self.option.work_dir.display(), v_file.build_metafile);
        let output_file = file_assembly::decode_file(
            &PathBuf::from(&metafile_path),
            PathBuf::from(&self.option.work_dir)
        ).unwrap();

        self.save_vfs().unwrap();

        Ok(PathBuf::from(format!(
            "{}{}.{}",
            self.option.work_dir.display(),
            v_file.name,
            v_file.extension
        )))
    }

    fn add_file_to_vfs(&self, separation_file: &SeparationFile, virtual_path: &Path) -> Result<(), VFSError> {
        let parts_name = separation_file.parts
            .iter()
            .map(|part| part.part_file_name.clone())
            .collect::<Vec<String>>();

        let metafile_name = separation_file.metafile.clone();

        let v_file = VFSFile {
            name: separation_file.filename.clone(),
            extension: separation_file.file_extension.clone(),
            build_metafile: metafile_name,
            parts_name,
            metadata: Default::default(),
        };

        let res = self.fs.borrow_mut().add_file(virtual_path, v_file);

        self.save_vfs().unwrap();

        return res;
    }

    pub fn remove_file(&self, path_file: &Path) -> Result<(), CloudError> {
        let res = self.fs
            .borrow_mut()
            .remove_node(path_file)
            .map_err(|e| e.into());

        self.save_vfs().unwrap();

        return res;
    }

    pub fn remove_folder(&self, path_file: &Path) -> Result<(), CloudError> {
        let res = self.fs
            .borrow_mut()
            .remove_node(path_file)
            .map_err(|e| e.into());

        self.save_vfs().unwrap();

        return res;
    }
}

// META файл именуется одинаково (при загрузке одинаковых файлов идет перезапись meta файла)
