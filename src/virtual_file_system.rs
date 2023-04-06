use std::collections::HashMap;
use std::fmt::{self, Display, Formatter, write};
use std::path::{Path, PathBuf};

use serde::{Serialize, Deserialize};
use serde_json::Error;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FSOption {
    version: i64,
    owner: String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Metadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemNode {
    File(VFSFile),
    Folder(VFSFolder)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VFSFile {
    pub name: String,
    pub extension: String,
    pub build_metafile: String,
    pub parts_name: Vec<String>,
    pub metadata : Metadata
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VFSFolder {
    pub name: String,
    pub metadata : Metadata,
    pub children: HashMap<String, FileSystemNode>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualFileSystem {
    dirs: HashMap<String, FileSystemNode>,
    options: FSOption,
}

impl VirtualFileSystem {

    pub fn new(options: FSOption) -> Self {
        Self {
            dirs: HashMap::from([
                (
                    "fs:".to_string(),
                    FileSystemNode::Folder(VFSFolder {
                        name: "Root".to_string(),
                        metadata: Default::default(),
                        children: HashMap::default(),
                    })
                )
            ]),
            options,
        }
    }

    /// Получение файла по вирутальному пути
    pub fn get_file(&self, path: &Path) -> Result<&VFSFile, VFSError> {
        let res_node = self.get_fs_node(path)?;

        return match res_node {
            FileSystemNode::File(file) => Ok(file),
            FileSystemNode::Folder(_) => return Err(VFSError::FileNotFound)
        }
    }

    /// Получение мутабельного файла по вирутальному пути
    pub fn get_mut_file(&mut self, path: &Path) -> Result<&mut VFSFile, VFSError> {

        let res_node = self.get_mut_fs_node(path)?;

        return match res_node {
            FileSystemNode::File(file) => Ok(file),
            FileSystemNode::Folder(_) => return Err(VFSError::FileNotFound)
        }
    }

    /// Получение папки по вирутальному пути
    pub fn get_folder(&self, path: &Path) -> Result<&VFSFolder, VFSError> {

        let res_node = self.get_fs_node(path)?;

        return match res_node {
            FileSystemNode::File(_) => Err(VFSError::FolderNotFound),
            FileSystemNode::Folder(folder) => Ok(folder)
        }
    }

    /// Получение мутабельной папки по вирутальному пути
    pub fn get_mut_folder(&mut self, path: &Path) -> Result<&mut VFSFolder, VFSError> {

        let res_node = self.get_mut_fs_node(path)?;

        return match res_node {
            FileSystemNode::File(_) => Err(VFSError::FolderNotFound),
            FileSystemNode::Folder(folder) => Ok(folder)
        }
    }

    /// Добавление файла по виртуальному пути
    pub fn add_file(&mut self, path: &Path, file: VFSFile) -> Result<(), VFSError> {

        let folder_for_add = self.get_mut_fs_node(path)?;

        return match folder_for_add {
            FileSystemNode::Folder(folder) => {

                if folder.children.contains_key(&file.name) {
                    return Err(VFSError::FileAlreadyExists);
                }

                folder.children.insert(
                    file.name.clone(),
                    FileSystemNode::File(file)
                );

                Ok(())
            }
            FileSystemNode::File { .. } => Err(VFSError::PathError)
        }
    }

    /// Добавление папки по виртуальному пути
    pub fn add_folder(&mut self, path: &Path, folder: VFSFolder) -> Result<(), VFSError> {
        let folder_for_add = self.get_mut_fs_node(path)?;

        return match folder_for_add {
            FileSystemNode::Folder(fol) => {

                if fol.children.contains_key(&folder.name) {
                    return Err(VFSError::FolderAlreadyExists);
                }

                fol.children.insert(
                    folder.name.clone(),
                    FileSystemNode::Folder(folder)
                );
                Ok(())
            }
            FileSystemNode::File { .. } => Err(VFSError::Test)
        }
    }

    /// Удаление узла у виртуального пути
    pub fn remove_node(&mut self, path: &Path) -> Result<(), VFSError> {
        let mut path = PathBuf::from(path);
        let remove_name = path
            .iter()
            .last()
            .ok_or(VFSError::PathError)?.to_string_lossy().to_string();
        path.pop();

        let folder = self.get_mut_folder(&path)?;

        folder.children.remove(&remove_name).ok_or(
            VFSError::NodeNotRemove(
                Box::new(VFSError::NodeNotFound)
            )
        )?;

        Ok(())
    }

    /// Получение мутабельного узла виртуального пути
    fn get_mut_fs_node(&mut self, path: &Path) -> Result<&mut FileSystemNode, VFSError> {

        let mut path_iter = path.into_iter();
        let mut output_node = self.dirs.get_mut("fs:");

        path_iter.next();

        for path_part in path_iter {
            let path_part = &*path_part.to_string_lossy();

            match output_node.ok_or(VFSError::PathError)? {

                &mut FileSystemNode::Folder (ref mut folder) => {
                    output_node = folder.children.get_mut(path_part);
                },

                _ => return Err(VFSError::PathError)
            }
        }

        return output_node.ok_or(VFSError::PathError);
    }

    /// Получение узла виртуального пути
    fn get_fs_node(&self, path: &Path) -> Result<&FileSystemNode, VFSError> {
        let mut path_iter = path.into_iter();
        let mut output_node = self.dirs.get("fs:");

        path_iter.next();

        for path_part in path_iter {

            let path_part = &*path_part.to_string_lossy();

            match output_node.ok_or(VFSError::PathError)? {

                &FileSystemNode::Folder (ref folder) => {
                    output_node = folder.children.get(path_part);
                },

                _ => return Err(VFSError::PathError)
            }
        }

        return output_node.ok_or(VFSError::PathError);
    }
}

impl Display for VirtualFileSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[derive(Debug)]
pub enum VFSError {
    Test,
    NodeNotFound,
    FolderNotFound,
    FileNotFound,
    NodeNotRemove(Box<dyn std::error::Error + 'static>),
    FileAlreadyExists,
    FolderAlreadyExists,
    PathError,
}

impl Display for VFSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for VFSError { }