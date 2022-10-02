//! Defines definitions and helpers for dealing with calendars.
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use tui::style::Color;
use tui::style::Style;

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
    /// Sets the style of all events in a calendar. Use an index
    /// between [0..4], after that, the default Style applies.
    pub fn set_event_style(&mut self, index: usize) {
        let style = get_style_from_idx(index);
        self.days
            .values_mut()
            .flat_map(|d| -> &mut Vec<Event> { d.events.as_mut() })
            .for_each(|e| e.style = style);
    }
    /// Reads a calendar from a given file path, and creates a
    /// calendar object.
    pub fn from_path(path: &str) -> Result<Self, std::io::Error> {
        let reader = BufReader::new(File::open(path)?);
        let parser = ical::PropertyParser::from_reader(reader);

        let mut events: Vec<Event> = vec![];
        let mut current: Event = Event::default();
        for l in parser
            .map(|l| l.expect("iCal compliant file"))
            .skip_while(|l| l.value.is_none())
        {
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

fn get_style_from_idx(index: usize) -> Style {
    match index {
        0 => Style::default()
            .bg(Color::Rgb(50, 50, 215))
            .fg(Color::White),
        1 => Style::default()
            .bg(Color::Rgb(190, 50, 50))
            .fg(Color::White),
        2 => Style::default()
            .bg(Color::Rgb(200, 110, 20))
            .fg(Color::White),
        3 => Style::default()
            .bg(Color::Rgb(150, 10, 200))
            .fg(Color::White),
        4 => Style::default()
            .bg(Color::Rgb(30, 175, 50))
            .fg(Color::White),
        _ => Style::default().bg(Color::White).fg(Color::Black),
    }
}

/// Determines the maximum height of this element.
pub trait MaxHeight {
    /// Get the max height, starting from a given date, up til
    /// a given number of days.    
    fn max_height(&self, from: usize, number: usize) -> u16;
}

impl MaxHeight for Vec<Calendar> {
    fn max_height(&self, from: usize, number: usize) -> u16 {
        assert!(number > 0);
        let mut max = 0u16;
        for i in 0..number {
            let date = (from + i).to_string();
            let day_max = self
                .iter()
                .map(|c| match c.days.get(&date) {
                    None => 0u16,
                    Some(day) => day.events.len() as u16,
                })
                .sum();
            max = max.max(day_max);
        }
        max
    }
}
