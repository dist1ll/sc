use std::io;

use clap::{Arg, ArgMatches, Command};

pub mod model;
use model::Calendar;
use render::{render_view_default, print_terminal};

pub mod render;

fn main() -> Result<(), io::Error> {
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
        Some(_) => println!("Unsupported command!"),
    };

    Ok(())
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
