#![warn(trivial_casts)]
#![deny(unused, unused_qualifications)]
#![forbid(unused_import_braces)]

extern crate itertools;

use std::{env, io};
use std::process::Command;

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

fn main_inner() -> Result<(), Error> {
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
    Ok(())
}

fn main() {
    main_inner().expect("error in open_all");
}
