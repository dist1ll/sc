use clap::{Arg, ArgMatches, Command};

pub mod files;

pub mod model;
use files::{init_config, Config};
use model::Calendar;
use render::{print_terminal, render_view_default};

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
    match m.subcommand_name() {
        None => cmd_view(m),
        Some("add") => cmd_add(m, &mut cfg),
        Some("remove") => println!("Removed calendar with ID"),
        Some("list") => cmd_list(&mut cfg),
        Some("update") => println!("Updating all calendars:"),
        Some(_) => panic!("Unsupported command!"),
    };

    Ok(())
}

/// Handles the list command
fn cmd_list(cfg: &mut Config) {
    for (idx, url) in cfg.get_urls().iter().enumerate() {
        let mut u = url.clone();
        let url_formatted = match u.len() > 50 {
            true => { u.truncate(50); (u + "...") },
            false => u,
        };
        println!("[{}] {}", idx, url_formatted);
    }
}

/// Handles the add command
fn cmd_add(m: ArgMatches, cfg: &mut Config) {
    let url = m
        .subcommand_matches("add")
        .unwrap()
        .get_one::<String>("url")
        .unwrap();
    cfg.add_line(url);
    cfg.save_config().expect("store changes to config file");
    println!("{:?}", cfg.get_urls());
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
