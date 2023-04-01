#![feature(buf_read_has_data_left)]
#![feature(file_create_new)]

pub mod file_separation;
pub mod file_assembly;

#[derive(Debug)]
pub struct FilePart {
    pub part_file: std::fs::File,
    pub hash_bytes: Vec<u8>,
    pub part_file_name: String,
}

#[derive(Debug)]
pub struct CompositeFile {
    pub filename: String,
    pub file_extension: String,
    pub file_len: usize,
    pub parts: Vec<FilePart>,
    pub uuid_parts: String,
}

pub struct Options {
    pub path_for_save: Option<std::path::PathBuf>,
    pub count_parts: Option<u8>,
    pub part_size: Option<usize>,
    pub compressed: Option<bool>,
}