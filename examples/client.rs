extern crate gopher;

use gopher::client::Gopher;

use std::io::IoResult;

fn stuff() -> IoResult<()> {
    let gopher = Gopher::new("freeshell.org", 70);
    let menu = try!(gopher.menu());
    for x in menu.iter() {
        println!("{}", x);
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
