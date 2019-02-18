#![warn(trivial_casts)]
#![deny(unused, unused_qualifications)]
#![forbid(unused_import_braces)]

use std::{
    env,
    io,
    process::Command,
    thread,
    time::Duration
};
use itertools::Itertools;

#[derive(Debug)]
enum Error {
    Io(io::Error),
    MissingPageId,
    MissingRevId,
    MissingWikiUrl
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

fn main() -> Result<(), Error> {
    let mut args = env::args();
    let _ = args.next(); // ignore executable name
    let wiki_url = args.next().ok_or(Error::MissingWikiUrl)?;
    let mut processes = Vec::default();
    for mut arg_pair in args.chunks(2).into_iter() {
        let pageid = arg_pair.next().ok_or(Error::MissingPageId)?;
        let old_revid = arg_pair.next().ok_or(Error::MissingRevId)?;
        processes.push(
            Command::new("open")
                .arg(format!("{}?pageid={}&diff=next&oldid={}", wiki_url, pageid, old_revid))
                .spawn()?
        );
    }
    for mut process in processes {
        process.wait()?;
    }
    thread::sleep(Duration::from_secs(2)); // wait for 2 seconds to allow for marking pages as visited before letting BitBar refresh the plugin
    Ok(())
}
