pub mod cloud;
pub mod virtual_file_system;
pub mod telegram_backend;

#[cfg(test)]
mod test {
    use std::path::Path;
    use crate::telegram_backend::TelegramBackend;
    use crate::virtual_file_system::{VFSFile, VFSFolder};
    use super::virtual_file_system::{FSOption, VirtualFileSystem};

    #[test]
    pub fn test_vfs() {
        let mut fs = VirtualFileSystem::new(FSOption::default());

        assert!(
            fs.add_file(Path::new("fs://"), VFSFile {
                name: "test".to_owned(),
                extension: "json".to_owned(),
                build_metafile: "build_this_file.meta".to_owned(),
                parts_name: vec!["1_parts.part".to_owned(), "2_parts.part".to_owned()],
                metadata: Default::default(),
            }).is_ok()
        );

        assert!(
            fs.add_folder(Path::new("fs://"), VFSFolder {
                name: "folder_test".to_owned(),
                metadata: Default::default(),
                children: Default::default(),
            }).is_ok()
        );

        assert!(
            fs.add_file(Path::new("fs://folder_test"), VFSFile {
                name: "test_file2".to_owned(),
                extension: "exe".to_owned(),
                build_metafile: "build_this_file.meta".to_owned(),
                parts_name: vec![
                    "1_parts.part".to_owned(),
                    "2_parts.part".to_owned(),
                    "2_parts.part".to_owned()
                ],
                metadata: Default::default(),
            }).is_ok()
        );

        assert!(serde_json::to_value(&fs).is_ok());

        assert!(fs.remove_node(Path::new("fs://folder_test")).is_ok());

        //println!("{:#}", &fs);
        //println!("{:#}", serde_json::to_value(&fs).unwrap());
    }

    #[test]
    fn tg_backend() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let telegram_backend = TelegramBackend::new().await;
        });
    }
}