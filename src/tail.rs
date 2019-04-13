///
/// This code originally sourced under the Apache Software License from
/// https://github.com/tdbgamer/Tail 
///
/// Copyright (c) 2018 Timothy Bess <tdbgamer@gmail.com>

use std::fs::{File, Metadata};
use std::io::{Seek, BufReader, SeekFrom, Read, BufWriter, Write};
use std::collections::{VecDeque};

const BUFFER_SIZE: u64 = 4096;

pub enum ModificationType {
    Added,
    Removed,
    NoChange,
}

#[derive(Debug)]
pub struct StatefulFile {
    pub fd: BufReader<File>,
    pub old_metadata: Metadata,
    file_name: String,
    cursor: SeekFrom,
}

impl StatefulFile {
    pub fn new(fd: File, file_name: String) -> Self {
        StatefulFile {
            old_metadata: fd.metadata()
                .unwrap_or_else(|_| { panic!("Could not retrieve metadata for file: {}", &file_name) }),
            fd: BufReader::new(fd),
            file_name: file_name,
            cursor: SeekFrom::Start(0),
        }
    }

    pub fn update_metadata(&mut self) {
        self.old_metadata = self.fd.get_ref().metadata()
            .unwrap_or_else(|_| { panic!("Could not retrieve metadata for file: {}", self.file_name) });
    }

    pub fn modification_type(&self) -> ModificationType {
        let new_metadata = self.fd.get_ref().metadata()
            .unwrap_or_else(|_| { panic!("Could not retrieve metadata for file: {}", self.file_name) });
        if new_metadata.len() > self.old_metadata.len() {
            ModificationType::Added
        } else if new_metadata.len() < self.old_metadata.len() {
            ModificationType::Removed
        } else {
            ModificationType::NoChange
        }
    }

    pub fn seek_to_cursor(&mut self) {
        self.fd.seek(self.cursor).unwrap();
    }

    pub fn update_cursor(&mut self) {
        self.cursor = SeekFrom::Start(self.fd.seek(SeekFrom::Current(0)).unwrap());
    }

    pub fn reset_cursor(&mut self) {
        self.cursor = SeekFrom::Start(0);
    }
}
