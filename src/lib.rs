use std::fmt::{self, Debug, Formatter};
use std::path::Path;
use std::{fs, io, str};

pub struct MboxReader<'a> {
    data: &'a MboxFile,
    idx: usize,
    prev: usize,
    testing: usize,
}

impl<'a> MboxReader<'a> {
    fn new(map: &MboxFile) -> MboxReader {
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
        if self.prev != self.idx {
            let entry = Entry {
                idx: self.idx,
                bytes: &bytes[self.prev..self.idx],
            };
            self.prev = self.idx;
            Some(entry)
        } else {
            None
        }
    }
}

/// The mbox file to read. This uses the OS facility to memory-map the file in
/// order to read it efficiently.
pub struct MboxFile {
    map: memmap::Mmap,
}

impl MboxFile {
    pub fn from_file(name: &Path) -> io::Result<MboxFile> {
        Ok(MboxFile {
            map: unsafe { memmap::Mmap::map(&fs::File::open(name)?)? },
        })
    }
    fn len(&self) -> usize {
        self.map.len()
    }
    fn as_slice(&self) -> &[u8] {
        &self.map
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
    pub fn offset(&self) -> usize {
        self.idx
    }
    pub fn start(&self) -> Start {
        match self.bytes.iter().position(|b| *b == b'\n') {
            Some(pos) => Start::new(&self.bytes[..pos - 1]),
            None => Start::new(&self.bytes),
        }
    }
    pub fn message(&self) -> Option<&[u8]> {
        self.bytes
            .iter()
            .position(|b| *b == b'\n')
            .and_then(|idx| Some(&self.bytes[idx + 1..]))
    }
}

impl<'a> Debug for Entry<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!(
            "Entry {{ {} bytes @ {} }}",
            self.bytes.len(),
            self.idx
        ))
    }
}

pub struct Start<'a> {
    bytes: &'a [u8],
    address: &'a str,
    date: &'a str,
}

impl<'a> Start<'a> {
    fn new(bytes: &'a [u8]) -> Start {
        let mut parts = bytes.splitn(3, |b| *b == b' ');
        let _ = parts.next();
        let address = str::from_utf8(parts.next().unwrap()).unwrap();
        let date = str::from_utf8(parts.next().unwrap()).unwrap();
        Start {
            bytes,
            address,
            date,
        }
    }
    pub fn address(&self) -> &str {
        self.address
    }
    pub fn date(&self) -> &str {
        self.date
    }
    pub fn as_str(&self) -> &str {
        str::from_utf8(self.bytes).unwrap()
    }
}
