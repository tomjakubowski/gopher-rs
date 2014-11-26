#![allow(dead_code, unused_variables)]

use std::fmt;
use std::io::{Reader, IoResult};

#[repr(u8)]
#[deriving(Show, PartialEq, Eq, FromPrimitive)]
pub enum KnownDirEntity {
    File = b'0',
    Dir  = b'1',
    CsoQuery = b'2',
    Error = b'3',
    MacBinHex = b'4',
    DosBin = b'5',
    Uuenc = b'6',
    SearchQuery = b'7',
    Telnet = b'8',
    Binary = b'9',
    RedundantServer = b'+',
    Tn3270 = b'T',
    Gif = b'g',
    Html = b'h',
    Info = b'i',
    Image = b'I'
}

#[deriving(Show, PartialEq, Eq)]
pub enum DirEntityKind {
    Known(KnownDirEntity),
    Unknown(u8)
}

impl DirEntityKind {
    fn from_byte(byte: u8) -> DirEntityKind {
        match FromPrimitive::from_u8(byte) {
            None => DirEntityKind::Unknown(byte),
            Some(item) => DirEntityKind::Known(item)
        }
    }
}

#[deriving(Show, PartialEq, Eq)]
pub struct DirEntity {
    pub kind: DirEntityKind,
    // FIXME: RFC 1436 allows (but does not recommend) Latin1 for this field, so
    // this should support that
    pub display: String,
    // Might be 0-length.
    pub selector: Vec<u8>,
    pub host: String,
    pub port: u16
}

/// Parses the Gopher protocol.
pub struct Parser<'a> {
    reader: Box<Reader + 'a>,
    lookahead: Option<u8>,
    byte: u8,
}

// utility function to convert ASCII bytes to a String
fn bytes_to_string(bytes: &[u8]) -> String {
    // FIXME: don't panic
    use std::ascii::{AsciiCast, AsciiStr};
    let ascii = bytes.to_ascii();
    ascii.as_str_ascii().into_string()
}

impl<'a> Parser<'a> {
    pub fn new<R: Reader + 'a>(reader: R) -> IoResult<Parser<'a>> {
        let mut parser = Parser {
            reader: box reader as Box<Reader>,
            lookahead: None,
            byte: 0,
        };
        try!(parser.bump());
        Ok(parser)
    }

    fn parse_direntity(&mut self) -> IoResult<DirEntity> {
        use std::str::FromStr;

        let kind = self.byte;
        try!(self.bump());
        let display = bytes_to_string(try!(self.parse_field()).as_slice());
        let selector = try!(self.parse_field());
        let host = bytes_to_string(try!(self.parse_field()).as_slice());
        let port = bytes_to_string(try!(self.parse_field()).as_slice());

        // Skip over any other remaining tab-delimited fields
        loop {
            if self.byte == b'\r' && try!(self.peek()) == b'\n' {
                try!(self.bump()); try!(self.bump());
                break;
            }
            try!(self.bump());
        }
        Ok(DirEntity {
            kind: DirEntityKind::from_byte(kind),
            display: display.into_string(),
            selector: selector,
            host: host.into_string(),
            port: FromStr::from_str(port.as_slice()).unwrap() // FIXME
        })
    }

    /// Parses a Gopher 'menu' listing into `DirEntity`s.
    pub fn parse_menu(&mut self) -> IoResult<Vec<DirEntity>> {
        let mut out = Vec::with_capacity(16);
        loop {
            if self.byte == b'.' {
                break;
            }
            out.push(try!(self.parse_direntity()));
        }
        Ok(out)
    }

    // Parses one tab-delimeted field
    fn parse_field(&mut self) -> IoResult<Vec<u8>> {
        let mut out = Vec::with_capacity(32);
        loop {
            let byte = self.byte;
            if byte == b'\t' {
                try!(self.bump());
                break;
            } else if byte == b'\r' {
                if try!(self.peek()) == b'\n' {
                    break;
                } else {
                    out.push(byte);
                }
            } else {
                out.push(byte);
            }
            try!(self.bump());
        }
        out.shrink_to_fit();
        Ok(out)
    }

    fn peek(&mut self) -> IoResult<u8> {
        match self.lookahead {
            Some(byte) => Ok(byte),
            None => {
                let byte = try!(self.reader.read_byte());
                self.lookahead = Some(byte);
                Ok(byte)
            }
        }
    }

    fn bump(&mut self) -> IoResult<()> {
        self.byte = match self.lookahead.take() {
            Some(byte) => byte,
            None => try!(self.reader.read_byte())
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{DirEntity, DirEntityKind, KnownDirEntity, Parser};
    use std::io::BufReader;

    fn fixture(bytes: &[u8]) -> Parser {
        Parser::new(BufReader::new(bytes)).unwrap()
    }

    const DATA: &'static [u8] = b"\
    0About internet Gopher\tStuff:About us\trawBits.micro.umn.edu\t70\r\n\
    1Around University of Minnesota\tZ,5692,AUM\tunderdog.micro.umn.edu\t70\r\n\
    1Microcomputer News & Prices\tPrices/\tpserver.bookstore.umn.edu\t70\r\n\
    1Courses, Schedules, Calendars\t\tevents.ais.umn.edu\t9120\r\n\
    1Student-Staff Directories\t\tuinfo.ais.umn.edu\t70\r\n\
    1Departmental Publications\tStuff:DP:\trawBits.micro.umn.edu\t70\r\n\
    .";

    #[test]
    fn test_field() {
        let line = b"1Courses, Schedules, Calendars\t\tevents.ais.umn.edu\t9120\r\n";
        let mut parser = fixture(line);

        assert_eq!(parser.parse_field(), Ok(b"1Courses, Schedules, Calendars".to_vec()));
        assert_eq!(parser.parse_field(), Ok(b"".to_vec()));
        assert_eq!(parser.parse_field(), Ok(b"events.ais.umn.edu".to_vec()));
        assert_eq!(parser.parse_field(), Ok(b"9120".to_vec()));
    }

    #[test]
    fn test_direntity() {
        let mut parser = fixture(DATA);

        let item = DirEntity {
            kind: DirEntityKind::Known(KnownDirEntity::File),
            display: "About internet Gopher".into_string(),
            selector: b"Stuff:About us".to_vec(),
            host: "rawBits.micro.umn.edu".into_string(),
            port: 70
        };
        assert_eq!(parser.parse_direntity(), Ok(item));

        let item = DirEntity {
            kind: DirEntityKind::Known(KnownDirEntity::Dir),
            display: "Around University of Minnesota".into_string(),
            selector: b"Z,5692,AUM".to_vec(),
            host: "underdog.micro.umn.edu".into_string(),
            port: 70
        };
        assert_eq!(parser.parse_direntity(), Ok(item));
    }

    #[test]
    fn test_menu() {
        let mut parser = fixture(DATA);

        let items = parser.parse_menu().unwrap();
        assert_eq!(items.len(), 6);

        let item = DirEntity {
            kind: DirEntityKind::Known(KnownDirEntity::File),
            display: "About internet Gopher".into_string(),
            selector: b"Stuff:About us".to_vec(),
            host: "rawBits.micro.umn.edu".into_string(),
            port: 70
        };
        assert_eq!(items[0], item);

        let item = DirEntity {
            kind: DirEntityKind::Known(KnownDirEntity::Dir),
            display: "Around University of Minnesota".into_string(),
            selector: b"Z,5692,AUM".to_vec(),
            host: "underdog.micro.umn.edu".into_string(),
            port: 70
        };
        assert_eq!(items[1], item);
    }
}

