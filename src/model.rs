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
    /// Sets the style of all events in a calendar.
    fn set_event_style(&mut self, style: Style) {
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
