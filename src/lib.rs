extern crate memmap;

use std::fmt::{self, Debug, Formatter};
use std::io;
use std::str;

pub struct MboxReader<'a> {
    data: &'a MboxData,
    idx: usize,
    prev: usize,
    testing: usize,
}

impl<'a> MboxReader<'a> {
    fn new(map: &MboxData) -> MboxReader {
        MboxReader {
            data: map,
            idx: 0,
            prev: 0,
            testing: 5,
        }
    }
}

impl<'a> Iterator for MboxReader<'a> {
    type Item = Entry<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.data.as_slice();
        if self.idx >= self.data.len() {
            return None;
        }
        for b in &bytes[self.idx..] {
            if *b == b'\n' {
                self.testing = 5;
                self.idx += 1;
                continue;
            } else if self.testing == 5 && *b == b'F' {
                self.testing = 4;
            } else if self.testing == 4 && *b == b'r' {
                self.testing = 3;
            } else if self.testing == 3 && *b == b'o' {
                self.testing = 2;
            } else if self.testing == 2 && *b == b'm' {
                self.testing = 1;
            } else if self.testing == 1 && *b == b' ' {
                self.testing = 0;
                let start = self.idx - 4;
                if start != 0 {
                    let entry = Entry {
                        idx: start,
                        bytes: &bytes[self.prev..start],
                    };
                    self.prev = start;
                    return Some(entry);
                }
            } else {
                self.testing = 0;
            }
            self.idx += 1;
        }
        None
    }
}

pub struct MboxData {
    map: memmap::Mmap,
}

impl MboxData {
    pub fn from_file(name: &str) -> io::Result<MboxData> {
        Ok(MboxData {
            map: memmap::Mmap::open_path(&name, memmap::Protection::Read)?,
        })
    }
    fn len(&self) -> usize {
        self.map.len()
    }
    fn as_slice(&self) -> &[u8] {
        unsafe { self.map.as_slice() }
    }
    pub fn iter<'a>(&'a self) -> MboxReader<'a> {
        MboxReader::new(self)
    }
}

pub struct Entry<'a> {
    idx: usize,
    bytes: &'a [u8],
}

impl<'a> Entry<'a> {
    pub fn start(&self) -> Start {
        match self.bytes.iter().position(|b| *b == b'\n') {
            Some(pos) => Start { bytes: &self.bytes[..pos + 1] },
            None => Start { bytes: self.bytes },
        }
    }
    pub fn message(&self) -> Option<&[u8]> {
        self.bytes.iter().position(|b| *b == b'\n').and_then(
            |idx| Some(&self.bytes[idx + 1..])
        )
    }
}

impl<'a> Debug for Entry<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("Entry {{ {} bytes @ {} }}", self.bytes.len(), self.idx))
    }
}

pub struct Start<'a> {
    bytes: &'a [u8],
}

impl<'a> Start<'a> {
    pub fn as_str(&self) -> &str {
        str::from_utf8(self.bytes).unwrap()
    }
}
