use std::{env, io::{self, Read, stdout, Write}, path::PathBuf, os::unix::net::UnixStream};

use clap::{Parser, Subcommand};

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let input = match cli.file {
        Files::Next => "next",
        Files::Prev => "prev",
        Files::PlayPause => "play-pause",
        Files::Title => "title",
        Files::NextPlayer => "next-player",
        Files::PrevPlayer => "prev-player",
    };
    let input = format!("waybar-{input}.sock");
    let input = PathBuf::from(env::var_os("XDG_RUNTIME_DIR").expect("runtime dir not set"))
        .join("mpris-player-info")
        .join(input);

    let mut input = UnixStream::connect(input)?;
    let mut output = stdout().lock();
    let mut buf = [0u8;1024];
    loop{
        let len = input.read(&mut buf)?;
        output.write_all(&buf[0..len])?;
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    file: Files,
}

#[derive(Subcommand, Clone)]
enum Files {
    Next,
    Prev,
    PlayPause,
    Title,
    NextPlayer,
    PrevPlayer,
}
