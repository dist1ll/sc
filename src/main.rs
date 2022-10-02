use std::{process::exit, thread::sleep, time::Duration};

use clap::{Arg, ArgMatches, Command};

pub mod files;

pub mod model;
use files::{init_config, Config};
use model::Calendar;
use render::{print_terminal, render_view_default};
use spinners::{Spinner, Spinners};

use crate::files::store_calendar;

pub mod render;

fn main() -> Result<(), std::io::Error> {
    // read config
    let mut cfg = init_config();

    let m = Command::new("sc")
        .subcommand(
            Command::new("add")
                .about("Add shared calendar by URL")
                .arg(Arg::new("url").help("URL of the shared calendar"))
                .arg_required_else_help(true),
        )
        .subcommand(Command::new("list").about("List all calendars"))
        .subcommand(Command::new("update").about("Updates all calendars"))
        .subcommand(
            Command::new("remove")
                .about("Delete calendar with given ID")
                .arg(Arg::new("id").help("ID of the shared calendar."))
                .arg_required_else_help(true),
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
    let result = match m.subcommand_name() {
        None => cmd_view(m),
        Some("add") => cmd_add(m, &mut cfg),
        Some("remove") => cmd_remove(m, &mut cfg),
        Some("list") => cmd_list(&mut cfg),
        Some("update") => cmd_update(&mut cfg),
        Some(_) => panic!("Unsupported command!"),
    };

    match result {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
    Ok(())
}

/// Handles the update command
fn cmd_update(cfg: &mut Config) -> Result<(), &'static str> {
    let mut sp = Spinner::new(Spinners::Dots9, "Updating calendars".into());
    for url in cfg.get_urls().iter() {
        let body = ureq::get(url.as_str())
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        sp.stop();
        println!("...DONE");
        store_calendar(body, url.clone()).map_err(|_| "couldn't write to calendar file")?;
    }
    Ok(())
}

/// Handles the remove command
fn cmd_remove(m: ArgMatches, cfg: &mut Config) -> Result<(), &'static str> {
    let id: usize = m
        .subcommand_matches("remove")
        .unwrap()
        .get_one::<String>("id")
        .unwrap()
        .parse()
        .map_err(|_| "calendar ID needs to be an integer")?;

    match cfg.remove_line(id) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
    cfg.save_config()
        .map_err(|_| "couldn't write changes to config")?;
    Ok(())
}

/// Handles the list command
fn cmd_list(cfg: &mut Config) -> Result<(), &'static str> {
    for (idx, url) in cfg.get_urls().iter().enumerate() {
        let mut u = url.clone();
        let url_formatted = match u.len() > 50 {
            true => {
                u.truncate(50);
                u + "..."
            }
            false => u,
        };
        println!("[{}] {}", idx, url_formatted);
    }
    Ok(())
}

/// Handles the add command
fn cmd_add(m: ArgMatches, cfg: &mut Config) -> Result<(), &'static str> {
    let url = m
        .subcommand_matches("add")
        .ok_or("no add command found")?
        .get_one::<String>("url")
        .ok_or("no url parameter given")?;
    cfg.add_line(url);
    cfg.save_config()
        .map_err(|_| "couldn't write changes to config")?;
    Ok(())
}

/// Handles the view command.
fn cmd_view(m: ArgMatches) -> Result<(), &'static str> {
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
    Ok(())
}
