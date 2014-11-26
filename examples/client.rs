#![feature(globs, if_let)]

extern crate gopher;

use gopher::DirEntity;
use gopher::client::Gopher;

use std::io::IoResult;

fn pretty_print(item: &DirEntity) {
    use gopher::EntityKind::{Known, Unknown};
    use gopher::KnownEntityKind::*;

    let kind = match item.kind {
        Known(File) => "file",
        Known(Dir)  => "dir",
        Known(CsoQuery) => "cso",
        Known(Error) => "err",
        Known(MacBinHex) => "binhex",
        Known(DosBin) => "dosbin",
        Known(Uuenc) => "uuenc",
        Known(SearchQuery) => "search",
        Known(Telnet) => "tel",
        Known(Binary) => "bin",
        Known(RedundantServer) => "server",
        Known(Tn3270) => "tn3270",
        Known(Gif) => "gif",
        Known(Html) => "html",
        Known(Info) => "info",
        Known(Image) => "img",
        Unknown(_) => "?"
    };
    println!("[{:>6}] {}", kind, item.display);
}

fn stuff() -> IoResult<()> {
    let gopher = Gopher::new("freeshell.org", 70);
    let menu = try!(gopher.root());

    if let Some(dir) = menu.iter().find(|&x| x.is_dir()) {
        println!("found dir, path = {}", String::from_utf8_lossy(&*dir.selector));
        for x in try!(gopher.fetch_dir(&*dir.selector)).iter() {
            pretty_print(x);
        }
    }

    Ok(())
}

fn main() {
    match stuff() {
        Err(e) => {
            let mut err = std::io::stdio::stderr();
            let _ = writeln!(&mut err, "error: {}", e);
            std::os::set_exit_status(1);
        }
        _ => {}
    }
}
