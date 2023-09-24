const _: () = assert!(cfg!(feature = "binary"));

use std::{ffi::OsString, fs::File};

use anyhow::Result;
use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{Shell, generate};

#[cfg(feature = "toggle_hide")]
mod toggle_hide;

#[cfg(feature = "info_base")]
mod info;

#[cfg(feature = "info_waybar_cat")]
mod waybar_cat;

#[cfg(feature = "hide_dbus_server")]
mod hide_server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone)]
enum Command {
    #[cfg(feature = "toggle_hide")]
    ///toggle hide status
    ToggleHide,
    #[cfg(feature = "hide_dbus_server")]
    ///run hide status server
    HideServer,
    #[cfg(feature = "info_waybar_cat")]
    ///cat waybar info from one of the sockets
    InfoWaybarCat {
        #[command(subcommand)]
        file: waybar_cat::Files,
    },
    #[cfg(feature = "info_base")]
    ///listen on dbus and write bar info
    Info {
        #[command(subcommand)]
        bar: info::Bars,
        #[arg(short = 'd', long)]
        hidden: bool,
    },
    ///generate autocomplete scripts
    Autocomplete{
        shell:Shell,
        output: OsString
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        #[cfg(feature = "toggle_hide")]
        Command::ToggleHide => toggle_hide::main(),
        #[cfg(feature = "hide_dbus_server")]
        Command::HideServer => hide_server::main(),
        #[cfg(feature = "info_waybar_cat")]
        Command::InfoWaybarCat { file } => waybar_cat::main(file),
        #[cfg(feature = "info_base")]
        Command::Info { bar, hidden } => info::main(bar, hidden),
        Command::Autocomplete { shell, output } => {
            generate(shell, &mut Cli::command(), "mpris-player-info", &mut File::create(output).expect("opening output"));
            Ok(())
        },
    }
}
