//! Render primitives to render different views.

use time::{self, OffsetDateTime};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Rect},
    style::Color,
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal, TerminalOptions, Viewport,
};

use crossterm::style::Stylize;

use crate::model::*;

/// Maximum number of default blocks that should be drawn, depending
/// on the terminal size.
fn max_default_blocks() -> u16 {
    crossterm::terminal::size().expect("Get terminal size").0 / 26
}

/// Returns a terminal, on which the default view has been rendered.
/// `days` is the number of days that should be rendered. If `days`
/// is zero, the day count will be determined automatically from the
/// size of the terminal.
pub fn render_view_default(cal: &Vec<Calendar>, days: usize) -> Terminal<impl Backend> {
    let mut term = build_default_terminal(cal);
    let mut f = term.get_frame();
    let max = if days == 0 {
        max_default_blocks() as usize
    } else {
        days
    };
    let chunks = Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .margin(1)
        .constraints(vec![Constraint::Ratio(1, max as u32); max as usize])
        .split(f.size());

    let start_date = today();
    for i in 0..max {
        let date = (start_date + i).to_string();
        match cal[0].days.get(&date) {
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
fn build_default_terminal(cal: &Vec<Calendar>) -> Terminal<impl Backend> {
    let stdout = std::io::stdout();
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
/// Returns a number representing a YYYYMMDD date
fn today() -> usize {
    let d = OffsetDateTime::now_utc();
    let m: u8 = d.month().into();
    let y = d.year();
    (d.day() as usize) + (m as usize * 100) + (y as usize * 10000)
}
