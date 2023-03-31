use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use serde::{Serialize, Deserialize};

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

    pub fn get_fs_as_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn print_fs(&self) {
        println!("{:#?}", self.dirs);
    }

    pub fn get_mut_file(&mut self, path: &Path) -> Result<&mut VFSFile, VFSError> {

        let res_node = self.get_mut_fs_node(path)?;

        return match res_node {
            FileSystemNode::File(file) => Ok(file),
            FileSystemNode::Folder(_) => return Err(VFSError::Test)
        }
    }

    pub fn get_mut_folder(&mut self, path: &Path) -> Result<&mut VFSFolder, VFSError> {

        let res_node = self.get_mut_fs_node(path)?;

        return match res_node {
            FileSystemNode::File(_) => Err(VFSError::Test),
            FileSystemNode::Folder(folder) => Ok(folder)
        }
    }

    pub fn add_file(&mut self, path: &Path, file: VFSFile) -> Result<(), VFSError> {

        let folder_for_add = self.get_mut_fs_node(path)?;

        return match folder_for_add {
            FileSystemNode::Folder(folder) => {

                if folder.children.contains_key(&file.name) {
                    return Err(VFSError::Test);
                }

                folder.children.insert(
                    file.name.clone(),
                    FileSystemNode::File(file)
                );

                Ok(())
            }
            FileSystemNode::File { .. } => Err(VFSError::Test)
        }
    }

    pub fn add_folder(&mut self, path: &Path, folder: VFSFolder) -> Result<(), VFSError> {
        let folder_for_add = self.get_mut_fs_node(path)?;

        return match folder_for_add {
            FileSystemNode::Folder(fol) => {
                fol.children.insert(
                    folder.name.clone(),
                    FileSystemNode::Folder(folder)
                );
                Ok(())
            }
            FileSystemNode::File { .. } => Err(VFSError::Test)
        }
    }

    fn get_mut_fs_node(&mut self, path: &Path) -> Result<&mut FileSystemNode, VFSError> {

        let mut path_iter = path.into_iter();
        let mut output_node = self.dirs.get_mut("fs:");

        path_iter.next();

        for path_part in path_iter {
            let path_part = &*path_part.to_string_lossy();

            match output_node.ok_or(VFSError::Test)? {

                &mut FileSystemNode::Folder (ref mut folder) => {
                    output_node = folder.children.get_mut(path_part);
                },

                _ => return Err(VFSError::Test) //return Some(file).ok_or(VFSError::Test)
            }
        }

        return output_node.ok_or(VFSError::Test);
    }

    fn get_fs_node(&mut self, path: &Path) -> Result<&FileSystemNode, VFSError> {
        return self.get_mut_fs_node(path).map(|node| &*node)
    }
}

#[derive(Debug)]
pub enum VFSError {
    Test
}