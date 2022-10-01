use std::{
    fs::{create_dir_all, File},
    io::{self, BufReader, Read, Write},
};

use clap::{Arg, ArgMatches, Command};

pub mod model;
use model::Calendar;
use render::{print_terminal, render_view_default};

pub mod render;

fn cfg_dir() -> Option<String> {
    let hd = dirs::home_dir().map(|p| Some(p.to_str()?.to_string()))??;
    Some(format!("{}/.config/sc/", hd))
}
fn cfg_path() -> Option<String> {
    let hd = dirs::home_dir().map(|p| Some(p.to_str()?.to_string()))??;
    Some(format!("{}/.config/sc/config", hd))
}

fn main() -> Result<(), io::Error> {
    // read config
    let (cfg, cfg_file) = read_or_create_config();

    let m = Command::new("sc")
        .subcommand(
            Command::new("add")
                .about("Add shared calendar by URL")
                .arg(Arg::new("url").help("URL of the shared calendar")),
        )
        .subcommand(Command::new("list").about("List all calendars"))
        .subcommand(Command::new("update").about("Updates all calendars"))
        .subcommand(
            Command::new("remove")
                .about("Delete calendar with given ID")
                .arg(Arg::new("id").help("ID of the shared calendar.")),
        )
        .author("Adrian Alic <contact@alic.dev>")
        .version(clap::crate_version!())
        .about("Command-line utility for viewing shared calendars")
        .arg(clap::arg!(-d --days <days> "Display events for the next n days"))
        .arg(clap::arg!(-t --today "Display all events for today in detail"))
        .args_conflicts_with_subcommands(true)
        .disable_help_subcommand(true)
        .get_matches();

    // check if config directory and file exists
    match m.subcommand_name() {
        None => cmd_view(m),
        Some("add") => println!("Added URL to calendar"),
        Some("remove") => println!("Removed calendar with ID"),
        Some("list") => println!("Listing all following calenders:"),
        Some("update") => println!("Updating all calendars:"),
        Some(_) => panic!("Unsupported command!"),
    };

    Ok(())
}

/// Reads the contents of the config file and returns the file handle.
/// If no config was found, creates a new config file.
pub fn read_or_create_config() -> (String, File) {
    let cfg_file = match File::options()
        .read(true)
        .write(true)
        .open(cfg_path().unwrap())
    {
        Ok(f) => f,
        Err(_) => create_config(),
    };
    let mut buf = String::new();
    BufReader::new(&cfg_file).read_to_string(&mut buf).unwrap();
    (buf, cfg_file)
}
/// Creates a config file. Only call this when you know that no
/// config file exists already.
fn create_config() -> File {
    println!("creating config file in ~/.config/sc");
    create_dir_all(cfg_dir().unwrap()).expect("create .config directory for sc");
    File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(cfg_path().unwrap())
        .expect("Create file in cfg directory")
}
/// Handles the view command.
fn cmd_view(m: ArgMatches) {
    // parse calendars
    let cal_paths = vec!["./local/calendar.ics"];
    let cals: Vec<Calendar> = cal_paths
        .iter()
        .map(|path| Calendar::from_path(path))
        .collect::<Result<_, _>>()
        .unwrap();
    //  display today view
    if m.get_flag("today") {
        println!("TODO: Display detailed today-view. ");
    }
    // display default view (with optional day param)
    else {
        let count = match m.get_one::<String>("days") {
            None => 0usize,
            Some(x) => x.parse().expect("<days> should be an integer."),
        };
        let mut term = render_view_default(&cals[0], count);
        print_terminal(&mut term);
    }
}
