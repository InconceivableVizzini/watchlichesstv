// SPDX-License-Identifier: ISC

use crate::lichess::LichessTV;
use curl::easy::Easy2;
use notcurses::{Notcurses};

pub mod lichess;

fn main() -> Result<(), curl::Error> {
    let mut nc = Notcurses::new().unwrap();
    let mut cli = nc.cli_plane().unwrap();

    let mut feed = Easy2::new(LichessTV::new(&mut cli));
    feed.get(true)?;
    feed.url("https://lichess.org/api/tv/feed")?;
    feed.perform()?;
    Ok(())
}
