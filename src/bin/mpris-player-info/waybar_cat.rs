use std::{env, io::{Read, stdout, Write}, path::PathBuf, os::unix::net::UnixStream};

use anyhow::{anyhow, Context};
use clap::Subcommand;

pub fn main(file:Files) -> anyhow::Result<()> {

    let input = match file {
        Files::Next => "next",
        Files::Prev => "prev",
        Files::PlayPause => "play-pause",
        Files::Title => "title",
        Files::NextPlayer => "next-player",
        Files::PrevPlayer => "prev-player",
    };
    let input = format!("waybar-{input}.sock");
    let input = PathBuf::from(env::var_os("XDG_RUNTIME_DIR").ok_or_else(||anyhow!("XDG_RUNTIME_DIR not set"))?)
        .join("mpris-player-info")
        .join(input);

    let mut input = UnixStream::connect(input).context("connecting to unix stream")?;
    let mut output = stdout().lock();
    let mut buf = [0u8;1024];
    loop{
        let len = input.read(&mut buf).context("reading from stream")?;
        output.write_all(&buf[0..len]).context("writing to stdout")?;
    }
}

#[derive(Subcommand, Clone)]
pub enum Files {
    Next,
    Prev,
    PlayPause,
    Title,
    NextPlayer,
    PrevPlayer,
}
