use std::{convert::Infallible, str::FromStr};

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Debug)]
pub struct ReservationConflict {
    _a: ReservationWindow,
    _b: ReservationWindow,
}

#[derive(Debug)]
pub struct ReservationWindow {
    _rid: String,
    _start: DateTime<Utc>,
    _end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(conflict) = s.parse() {
            Ok(ReservationConflictInfo::Parsed(conflict))
        } else {
            Ok(ReservationConflictInfo::Unparsed(s.to_string()))
        }
    }
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
