use std::{
    error::Error,
    fs,
    io::{self, BufRead, Seek},
};

use super::progress;

pub struct TableReader {
    insert_prefix: String,
    table_name: String,
    reader: io::BufReader<fs::File>,
    buffer: String,
    file_len: u64,
    indexed_lines: u64,
}

impl TableReader {
    pub fn new(path: &str, table_name: &str) -> Result<Self, Box<dyn Error>> {
        let file = fs::File::open(path)?;
        let file_len = file.metadata()?.len();
        let reader = io::BufReader::new(file);
        Ok(Self {
            insert_prefix: format!("INSERT INTO `{}` VALUES ", table_name),
            table_name: table_name.to_string(),
            reader,
            buffer: String::new(),
            file_len,
            indexed_lines: 0,
        })
    }
    pub fn index_lines(&mut self) {
        let pb = progress::create(self.file_len, true);
        pb.set_message(format!("Indexing `{}` dump", self.table_name));

        let mut count = 0;
        while self.read_line() {
            pb.inc(self.buffer.len() as u64);
            if self.buffer.starts_with(&self.insert_prefix) {
                count += 1;
            }
        }
        self.reader.rewind().unwrap();

        pb.finish_with_message(format!(
            "Indexed {} lines from `{}` dump",
            count, self.table_name
        ));
        self.indexed_lines = count;
    }
    fn read_line(&mut self) -> bool {
        self.buffer.clear();
        if let Ok(len) = self.reader.read_line(&mut self.buffer) {
            len > 0
        } else {
            false
        }
    }
    pub fn len(&self) -> u64 {
        self.indexed_lines
    }
}

impl Iterator for TableReader {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while self.read_line() {
            if self.buffer.starts_with(&self.insert_prefix) {
                let slice = &self.buffer[self.insert_prefix.len()..self.buffer.len()];
                return Some(String::from(slice));
            }
        }
        None
    }
}
