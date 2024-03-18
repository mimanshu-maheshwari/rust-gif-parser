use std::fs::File;
use std::io::{BufReader, Read};
use std::os::windows::fs::MetadataExt;

#[derive(Debug, PartialEq)]
pub struct GifBuffer {
    buffer: Box<[u8]>,
    size: usize,
    pointer: usize,
}

impl GifBuffer {
    pub fn read(file_path: &str) -> Self {
        println!("INFO: Loading file {file_path}...");
        let mut file = File::open(&file_path)
            .map_err(|err| {
                eprintln!("ERROR: Unable to open file: {file_path}, due to error: {err}");
            })
            .unwrap();

        println!("INFO: Reading Metadata...");
        let metadata = file.metadata().expect("ERROR: Unable to parse metadata");
        let file_size: usize = metadata.file_size() as usize;

        println!("INFO: Reading data into buffer...");
        let mut b_reader: BufReader<&mut File> = BufReader::new(&mut file);
        let mut buf: Vec<u8> = vec![0u8; file_size];

        b_reader
            .read(&mut buf)
            .map_err(|err| {
                eprintln!("ERROR: Unable to read binary data into buffer: {err}");
            })
            .unwrap();
        println!("INFO: {size} bytes read into buffer.", size = buf.len());

        GifBuffer {
            buffer: buf.into_boxed_slice(),
            size: file_size,
            pointer: 0,
        }
    }

    pub fn get_pointer(&self) -> usize {
        self.pointer
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn read_u8(&mut self) -> u8 {
        let sl = self.buffer[self.pointer].to_owned();
        self.pointer += 1;
        sl
    }

    pub fn read_le_u16(&mut self) -> u16 {
        (self.read_u8() as u16) | ((self.read_u8() as u16) << 8)
    }

    pub fn read_u32(&mut self) -> u32 {
        ((self.read_u16() as u32) << 15) | (self.read_u16() as u32)
    }

    pub fn read_u16(&mut self) -> u16 {
        ((self.read_u8() as u16) << 7) | (self.read_u8() as u16)
    }
    pub fn skip_u8(&mut self) {
        self.pointer += 1;
    }
    pub fn peek_u8(&self) -> u8 {
        self.buffer[self.pointer]
    }
    pub fn read_slice(&mut self, bytes: usize) -> Vec<u8> {
        let sl = self.buffer[self.pointer..self.pointer + bytes].to_owned();
        self.pointer += bytes;
        sl
    }
}
