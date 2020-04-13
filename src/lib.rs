#![deny(clippy::all)]
#![deny(clippy::pedantic)]
extern crate chrono;

use chrono::{NaiveDate, NaiveTime};
use log::debug;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
enum TimeError {
    DateParse(chrono::ParseError),
    TimeParse(chrono::ParseError),
    ParseError(chrono::ParseError),
    TimeNotFound(String),
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TimeError::DateParse(ref err)
            | TimeError::TimeParse(ref err)
            | TimeError::ParseError(ref err) => err.fmt(f),
            TimeError::TimeNotFound(ref s) => write!(f, "{}", s),
        }
    }
}

impl Error for TimeError {}

impl From<chrono::ParseError> for TimeError {
    fn from(err: chrono::ParseError) -> TimeError {
        TimeError::ParseError(err)
    }
}

pub struct Time {
    pub entries: HashMap<NaiveDate, Vec<TimeEntry>>,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for (date, entries) in &self.entries {
            output.push_str(&format!("{}\n", date.format("%Y-%m-%d")));

            for e in entries {
                output.push_str(&format!("{}\n", e));
            }
        }

        write!(f, "{}", output)
    }
}

#[allow(dead_code)] // allow date
pub struct TimeEntry {
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub description: String,
}

struct Duration {
    start: NaiveTime,
    end: NaiveTime,
}

impl fmt::Display for TimeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.start.format("%H:%M").to_string(),
            self.end.format("%H:%M").to_string(),
            self.description
        )
    }
}

/// Constructs a new Time struct typically from a config file
/// the contents expect a specific time.txt format
///
/// Ex.
/// 1822-01-15
/// 03:00 04:00 Sketched ideas for a new machine
/// 04:00 11:00 Created the first computer
/// 15:30 17:30 Decided on the name, Difference Engine
///
/// # Errors
/// If the string doesn't fit the time.txt format it will error.
/// Common reasons are incorrect date and time format
pub fn parse_time(contents: &str) -> Result<Time, Box<dyn Error>> {
    let mut t = Time {
        entries: HashMap::new(),
    };

    let mut date: Option<NaiveDate> = None;
    for line in contents.lines() {
        debug!("line {}", line);
        // Ignore all lines that start with // as they are comments
        // or if they are empty
        if line.starts_with("//") || line.is_empty() {
            continue;
        }

        // Time and date entries are longer than 9 characters
        // Time 1:30 or 01:30, but always universal time
        // Date 1970-01-01
        if line.len() < 9 {
            continue;
        }

        // Check if the line is a date to indicate the start of a
        // date block
        let d = NaiveDate::parse_from_str(line, "%Y-%m-%d").ok();
        if let Some(d) = d {
            date = Some(d);
            continue; // found a date so proceed to next line
        }

        let (index, duration) = find_duration(line)?;
        let desc = &line[index..line.len()];
        let desc = desc.trim();

        // Time uses a hash map to sort entries by date
        date.and_then(|d| {
            let entry = TimeEntry {
                date: d,
                start: duration.start,
                end: duration.end,
                description: desc.to_string(),
            };

            // Check to see if it has the date key
            // if it doesn't add the key and create the
            // entries Vec
            let entries = t.entries.entry(d).or_insert_with(|| vec![]);
            entries.push(entry);

            Some(d)
        });
    }

    Ok(t)
}

fn find_duration(line: &str) -> Result<(usize, Duration), TimeError> {
    // The start date and end date are allows at the beginning of a line
    // and are separated by a space. Let's make sure we have two spaces
    let mut num_of_spaces = 0;
    let mut start_time: Option<NaiveTime> = None;
    let mut end_time: Option<NaiveTime> = None;
    let mut start_time_space = 0;
    let mut end_time_space = 0;
    for (i, c) in line.chars().enumerate() {
        if c == ' ' {
            num_of_spaces += 1;

            // If we have one space check for start date
            if num_of_spaces == 1 && start_time_space == 0 {
                // make sure i is greater than min
                // min is H:MM (4 characters)
                if i < 4 {
                    return Err(TimeError::TimeNotFound("Start time not found".to_string()));
                }

                // Make sure it's a valid time
                start_time_space = i;
                let st = NaiveTime::parse_from_str(&line[0..start_time_space], "%H:%M")?;
                start_time = Some(st);
            }

            // If we have two spaces check for start date
            if num_of_spaces == 2 && end_time_space == 0 {
                // make sure i is greater than min
                // min is H:MM H:MM (9 characters)
                if i < 9 {
                    return Err(TimeError::TimeNotFound("End time not found".to_string()));
                }

                // Make sure it's a valid time
                end_time_space = i;
                let et =
                    NaiveTime::parse_from_str(&line[start_time_space..end_time_space], "%H:%M")?;
                end_time = Some(et);
            }

            // After we found two spaces we "should" have both start and end time
            // Stop looking if we've read more than 12 characters and haven't
            // found two spaces. The format dicates a max of HH:MM HH:MM
            if num_of_spaces > 2 || i > 11 {
                break;
            }
        }
    }

    // If we have less than two spaces then we know we didn't
    // find a start or end date so continue
    if num_of_spaces < 2 {
        Err(TimeError::TimeNotFound(
            "Neither start or end time not found".to_string(),
        ))
    } else {
        let st: NaiveTime;
        let et: NaiveTime;
        match start_time {
            Some(t) => st = t,
            None => return Err(TimeError::TimeNotFound("Start time not found".to_string())),
        }

        match end_time {
            Some(t) => et = t,
            None => return Err(TimeError::TimeNotFound("End time not found".to_string())),
        }

        Ok((end_time_space, Duration { start: st, end: et }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_lines() -> Result<(), Box<dyn Error>> {
        let input = "1822-01-15\n\
        // Some comments\n\
        3:00 4:00 Sketched ideas for a new machine\n\
        4:00 11:00 Created the first computer\n\
        15:30 17:30 Decided on the name, Difference Engine\n";

        let expected = "1822-01-15\n\
        03:00 04:00 Sketched ideas for a new machine\n\
        04:00 11:00 Created the first computer\n\
        15:30 17:30 Decided on the name, Difference Engine\n";

        // Parse time from string
        let t = parse_time(input);
        match t {
            Ok(t) => {
                let result = format!("{}", t);
                assert_eq!(result, expected);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}
