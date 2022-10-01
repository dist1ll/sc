use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
};

use clap::{Arg, ArgMatches, Command};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal, TerminalOptions, Viewport,
};

use crossterm::style::Stylize;
use time::{self, OffsetDateTime};

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
        .arg(clap::arg!(-t --today "Show calendar events for today"))
        .get_matches();

    // check if config directory and file exists
    match m.subcommand_name() {
        None => cmd_view(),
        Some("add") => println!("Added URL to calendar"),
        Some("remove") => println!("Removed calendar with ID"),
        Some("list") => println!("Listing all following calenders:"),
        Some("update") => println!("Updating all calendars:"),
        Some(_) => println!("Unsupported command!"),
    };

    Ok(())
}

/// Handles the view command.
fn cmd_view() {
    let cal_paths = vec!["./local/calendar.ics"];
    let cals: Vec<Calendar> = cal_paths
        .iter()
        .map(|path| Calendar::from_path(path))
        .collect::<Result<_, _>>()
        .unwrap();
    let mut term = render_view_default(&cals[0]);
    print_terminal(&mut term);
}

#[derive(Default, Debug)]
pub struct Calendar {
    pub days: HashMap<String, Day>,
}

#[derive(Default, Debug, Clone)]
pub struct Day {
    pub date: String,
    pub events: Vec<Event>,
}

#[derive(Default, Debug, Clone)]
pub struct Event {
    pub name: String,
    pub description: String,
    pub date: String,
    pub style: Style,
}

impl Calendar {
    ///  Creates a calendar from an iterator of events  
    fn from_events<'a, T: Iterator<Item = &'a Event>>(events: T) -> Self {
        let mut cal = Calendar::default();
        for e in events {
            match cal.days.get_mut(&e.date) {
                None => {
                    let mut d = Day::default();
                    d.date = e.date.clone();
                    d.events.push(e.clone());
                    cal.days.insert(e.date.clone(), d);
                }
                Some(day) => day.events.push(e.clone()),
            };
        }
        cal
    }
    /// Sets the style of all events in a calendar.
    fn set_event_style(&mut self, style: Style) {
        self.days
            .values_mut()
            .flat_map(|d| -> &mut Vec<Event> { d.events.as_mut() })
            .for_each(|e| e.style = style);
    }
    /// Reads a calendar from a given file path, and creates a
    /// calendar object.
    pub fn from_path(path: &str) -> Result<Self, io::Error> {
        let reader = BufReader::new(File::open(path)?);
        let parser = ical::PropertyParser::from_reader(reader);

        let mut events: Vec<Event> = vec![];
        let mut current: Event = Event::default();
        for l in parser.map(|l| l.unwrap()).skip_while(|l| l.value.is_none()) {
            let val = l.value.as_ref().unwrap().clone();
            if val == "VEVENT" {
                match l.name.as_str() {
                    "BEGIN" => {
                        current = Event::default();
                        current.style = Style::default()
                            .bg(Color::Rgb(255, 255, 255))
                            .fg(Color::Rgb(0, 0, 0));
                    }
                    "END" => events.push(current.clone()),
                    _ => continue,
                }
            }
            match l.name.as_str() {
                "SUMMARY" => current.name = val,
                "DESCRIPTION" => current.description = val,
                "DTSTART" => {
                    let mut time_iter = val.split('T');
                    current.date = time_iter
                        .next()
                        .expect("DTSTART time string separated by T")
                        .to_string();
                }
                _ => continue,
            };
        }
        Ok(Self::from_events(events.iter()))
    }
    /// Get the max height, starting from a given date, up til
    /// a given number of days.
    pub fn max_height(&self, from: usize, number: usize) -> u16 {
        assert!(number > 0);
        let mut max = 0u16;
        for i in 0..number {
            let date = (from + i).to_string();
            match self.days.get(&date) {
                None => continue,
                Some(day) => {
                    max = max.max(day.events.len() as u16);
                }
            };
        }
        max
    }
}

/// Returns a number representing a YYYYMMDD date
pub fn today() -> usize {
    let d = OffsetDateTime::now_utc();
    let m: u8 = d.month().into();
    let y = d.year();
    (d.day() as usize) + (m as usize * 100) + (y as usize * 10000)
}

/// Maximum number of default blocks that should be drawn, depending
/// on the terminal size.
pub fn max_default_blocks() -> u16 {
    crossterm::terminal::size().expect("Get terminal size").0 / 26
}

/// Returns a terminal, on which the default view has been rendered. 
pub fn render_view_default(cal: &Calendar) -> Terminal<impl Backend> {
    let mut term = build_default_terminal(&cal);
    let mut f = term.get_frame();
    let max = max_default_blocks() as usize;
    let chunks = Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .margin(1)
        .constraints(vec![Constraint::Ratio(1, max as u32); max])
        .split(f.size());

    let start_date = today();
    for i in 0..max {
        let date = (start_date + i).to_string();
        match cal.days.get(&date) {
            None => {
                let mut day = Day::default();
                day.date = date;
                render_default_block(&mut f, chunks[i], &day);
            }
            Some(day) => render_default_block(&mut f, chunks[i], day),
        };
    }
    term
}

/// Builds terminal for default view from a given calendar.
pub fn build_default_terminal(cal: &Calendar) -> Terminal<impl Backend> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let ts = crossterm::terminal::size().unwrap();
    let max_blocks = max_default_blocks() as usize;
    let mut max_height = cal.max_height(today(), max_blocks);
    // 4 lines to account for lines added by block decorations.
    max_height += 4;
    // never longer than the terminal
    max_height = max_height.min(ts.1 - 1);

    Terminal::with_options(
        backend,
        TerminalOptions {
            // height - 1 to leave space for bottom command line
            viewport: Viewport::fixed(Rect::new(0, 0, ts.0, max_height)),
        },
    )
    .unwrap()
}
/// Renders a default overview block, with truncated event names
fn render_default_block<T: Backend>(f: &mut Frame<T>, pos: Rect, d: &Day) {
    let block = Block::default().borders(Borders::ALL).title(d.date.clone());
    render_vertical_paragraphs(f, block.inner(pos), d.events.iter().collect());
    f.render_widget(block, pos);
}

/// Renders a collection of single-line texts in a given Rect.
fn render_vertical_paragraphs<T: Backend>(f: &mut Frame<T>, pos: Rect, text: Vec<&Event>) {
    for (i, &e) in text.iter().enumerate() {
        let mut l = pos.clone();
        l.height = 1;
        l.y += i as u16;
        if l.y >= pos.bottom() {
            break;
        }
        let p = Paragraph::new(Text::from(e.name.as_str())).style(e.style);
        f.render_widget(p, l);
    }
}

pub fn print_terminal<T: Backend>(t: &mut Terminal<T>) {
    let buffer = t.current_buffer_mut();
    for (_, val) in buffer.content.iter().enumerate() {
        let styled = val
            .symbol
            .clone()
            .with(conv_color(val.fg))
            .on(conv_color(val.bg));
        print!("{}", styled);
    }
}

pub fn conv_color(c1: tui::style::Color) -> crossterm::style::Color {
    type Cs = crossterm::style::Color;
    match c1 {
        Color::Rgb(r, g, b) => Cs::Rgb { r, g, b },
        Color::White => Cs::White,
        Color::Black => Cs::Black,
        Color::Reset => Cs::Reset,
        Color::Indexed(x) => Cs::AnsiValue(x),
        _ => Cs::Black,
    }
}
