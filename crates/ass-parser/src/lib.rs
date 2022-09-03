use std::{convert::Infallible, io::BufRead, str::FromStr};

use chrono::NaiveTime;
use indexmap::IndexMap;

impl From<Event> for zimu_ast::Block {
    fn from(x: Event) -> Self {
        Self {
            start: x.start,
            end: x.end,
            content: x.text,
        }
    }
} 

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum StyleFormatKey {
    Name,
    Fontname,
    Fontsize,
    PrimaryColour,
    SecondaryColour,
    OutlineColour,
    BackColour,
    Bold,
    Italic,
    Underline,
    StrikeOut,
    ScaleX,
    ScaleY,
    Spacing,
    Angle,
    BorderStyle,
    Outline,
    Shadow,
    Alignment,
    MarginL,
    MarginR,
    MarginV,
    Encoding,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventFormatKey {
    Layer,
    Start,
    End,
    Style,
    Name,
    MarginL,
    MarginR,
    MarginV,
    Effect,
    Text,
    Unknown(String),
}

impl FromStr for EventFormatKey {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Layer" => Self::Layer,
            "Start" => Self::Start,
            "End" => Self::End,
            "Style" => Self::Style,
            "Name" => Self::Name,
            "MarginL" => Self::MarginL,
            "MarginR" => Self::MarginR,
            "MarginV" => Self::MarginV,
            "Effect" => Self::Effect,
            "Text" => Self::Text,
            unknown => Self::Unknown(unknown.to_string()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Style {}

#[derive(Debug, Clone)]
pub struct AssFile {
    pub script_info: IndexMap<String, String>,
    pub styles: Vec<Style>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub text: Vec<String>,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub meta: IndexMap<EventFormatKey, String>,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    ExpectSection,
    Section(String),
}

pub fn parse<R: BufRead>(input: R) -> AssFile {
    let mut state = State::ExpectSection;
    let mut styles_buf = String::new();
    let mut event_format = vec![];
    let mut events: Vec<Event> = vec![];

    for line in input.lines() {
        let line = line.expect("invalid line");
        let line = line.trim();

        if line.trim() == "" {
            state = State::ExpectSection;
            continue;
        }

        match state {
            State::ExpectSection => {
                if line.starts_with("[") && line.ends_with("]") {
                    let section = line.trim_matches(|c| c == '[' || c == ']');

                    state = State::Section(section.to_string());
                    continue;
                }
            }
            State::Section(ref s) if s.ends_with("Styles") => {
                if line.starts_with("Format: ") {
                    let line = line.trim_start_matches("Format: ");
                    styles_buf.push_str(line);
                    styles_buf.push('\n');
                }
                if line.starts_with("Style: ") {
                    let line = line.trim_start_matches("Style: ");
                    styles_buf.push_str(line);
                    styles_buf.push('\n');
                }
            }
            State::Section(ref s) if s == "Events" => {
                if line.starts_with("Format: ") {
                    let line = line.trim_start_matches("Format: ");
                    event_format = line
                        .split(",")
                        .map(str::trim)
                        .map(|x| x.parse::<EventFormatKey>().unwrap())
                        .collect();
                    continue;
                }
                if line.starts_with("Dialogue: ") {
                    let line = line.trim_start_matches("Dialogue: ");
                    if event_format.is_empty() {
                        panic!("No format line found.");
                    }

                    let pieces = line.splitn(event_format.len(), ",");
                    let mut meta = event_format
                        .iter()
                        .cloned()
                        .zip(pieces.map(|x| x.to_string()))
                        .collect::<IndexMap<_, _>>();

                    let text = meta
                        .remove(&EventFormatKey::Text)
                        .unwrap()
                        .split("\\N")
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    let start = NaiveTime::parse_from_str(
                        &meta.remove(&EventFormatKey::Start).unwrap(),
                        "%H:%M:%S%.f",
                    )
                    .unwrap();
                    let end = NaiveTime::parse_from_str(
                        &meta.remove(&EventFormatKey::End).unwrap(),
                        "%H:%M:%S%.f",
                    )
                    .unwrap();

                    events.push(Event {
                        text,
                        start,
                        end,
                        meta,
                    });
                }
            }
            State::Section(_) => {}
        }
    }

    AssFile {
        script_info: Default::default(),
        styles: Default::default(),
        events,
    }
}
