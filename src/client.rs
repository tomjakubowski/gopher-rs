use std::io::IoResult;
use std::io::net::tcp::TcpStream;

use protocol::{DirEntity, Parser};

/// A client which knows the host and port of a Gopher server.  Due to the
/// ephemeral nature of the Gopher protocol, each operation on a Gopher creates
/// a connection to the server and disconnects after running.
pub struct Gopher {
    host: String,
    port: u16
}

impl Gopher {
    /// Constructs a new Gopher given some server's address.
    pub fn new(host: &str, port: u16) -> Gopher {
        Gopher {
            host: host.into_string(),
            port: port
        }
    }

    /// List all entries at the root of the server.
    pub fn menu(&self) -> IoResult<Vec<DirEntity>> {
        use std::io::BufferedReader;

        let mut stream = try!(TcpStream::connect((self.host.as_slice(), self.port)));
        try!(stream.write(b"\r\n"));
        let mut parser = try!(Parser::new(BufferedReader::new(stream)));
        parser.parse_menu()
    }
}
