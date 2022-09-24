use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
};

use chrono::Local;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal, TerminalOptions, Viewport,
};

use crossterm::style::{style, Stylize};

fn main() -> Result<(), io::Error> {
    let reader = BufReader::new(File::open("./local/calendar.ics")?);
    let parser = ical::PropertyParser::from_reader(reader);

    let mut events: Vec<Event> = vec![];
    let mut current: Event = Event::default();

    for l in parser.map(|l| l.unwrap()).skip_while(|l| l.value.is_none()) {
        let val = l.value.as_ref().unwrap().clone();
        if val == "VEVENT" {
            match l.name.as_str() {
                "BEGIN" => current = Event::default(),
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

    let c = Calendar::from_events(events.into_iter());

    let mut terminal = build_default_terminal(&c);
    let mut frame = terminal.get_frame();

    render_view_default(&mut frame, c);
    print_terminal(&mut terminal);

    Ok(())
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
}

impl Calendar {
    ///  Creates a calendar from an iterator of events  
    fn from_events<T: Iterator<Item = Event>>(events: T) -> Self {
        let mut cal = Calendar::default();
        for e in events {
            match cal.days.get_mut(&e.date) {
                None => {
                    let mut d = Day::default();
                    d.date = e.date.clone();
                    d.events.push(e.clone());
                    cal.days.insert(e.date, d);
                }
                Some(day) => day.events.push(e.clone()),
            };
        }
        cal
    }

    /// Get the max height, starting from a given date, up til
    /// a given number of days.
    fn max_height(&self, from: usize, number: usize) -> u16 {
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
fn today() -> usize {
    let dt = Local::now();
    dt.date()
        .to_string()
        .split('+')
        .next()
        .unwrap()
        .replace("-", "")
        .parse()
        .expect("Parse chrono datestring to u16")
}

/// Maximum number of default blocks that should be drawn, depending
/// on the terminal size.
fn max_default_blocks() -> u16 {
    crossterm::terminal::size().expect("Get terminal size").0 / 26
}

/// Builds terminal for default view from a given calendar.
fn build_default_terminal(cal: &Calendar) -> Terminal<impl Backend> {
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

/// Renders the short event overview, with one block per day.
fn render_view_default<T: Backend>(f: &mut Frame<T>, cal: Calendar) {
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
                render_default_block(f, chunks[i], &day);
            }
            Some(day) => render_default_block(f, chunks[i], day),
        };
    }
}

/// Renders a default overview block, with truncated event names
fn render_default_block<T: Backend>(f: &mut Frame<T>, pos: Rect, d: &Day) {
    let block = Block::default().borders(Borders::ALL).title(d.date.clone());
    let ev = d.events.iter().map(|e| e.name.as_str()).collect();
    render_vertical_paragraphs(f, block.inner(pos), ev);
    f.render_widget(block, pos);
}

/// Renders a collection of single-line texts in a given Rect.
fn render_vertical_paragraphs<T: Backend>(f: &mut Frame<T>, pos: Rect, text: Vec<&str>) {
    for (i, &e) in text.iter().enumerate() {
        let mut l = pos.clone();
        l.height = 1;
        l.y += i as u16;
        if l.y >= pos.bottom() {
            break;
        }
        let p = Paragraph::new(Text::from(e)).style(
            Style::default()
                .bg(Color::Rgb(20, 20, 200))
                .fg(Color::Rgb(255, 255, 255)),
        );
        f.render_widget(p, l);
    }
}

fn print_terminal<T: Backend>(t: &mut Terminal<T>) {
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

fn conv_color(c1: tui::style::Color) -> crossterm::style::Color {
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
