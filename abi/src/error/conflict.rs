use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

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

/// error message
///Key (resource_id, rperiod)=(ocean roon-745, [\"2022-11-04 07:00:00+00\",\"2022-11-08 04:00:00+00\"))
/// conflicts with existing
/// key (resource_id, rperiod)=(ocean roon-745, [\"2022-11-01 07:00:00+00\",\"2022-11-07 04:00:00+00\"))
impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

#[allow(dead_code)]
struct ParsedInfo {
    a: HashMap<String, String>,
    b: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // \((?P<k1>[a-zA-Z0-9_-]+),\s*(?P<k2>[a-zA-Z0-9]+)\)=\((?P<v1>[a-zA-Z0-9_-]+\s*[a-zA-Z0-9_-]+),\s*(?P<v2>\[[^\)\]]+)
        let re = Regex::new(
            r"\((?P<k1>[a-zA-Z0-9_-]+),\s*(?P<k2>[a-zA-Z0-9]+)\)=\((?P<v1>[a-zA-Z0-9_-]+\s*[a-zA-Z0-9_-]+),\s*(?P<v2>\[[^\)\]]+)",
        ).unwrap();
        let mut result = vec![];
        for caps in re.captures_iter(s) {
            let mut map = HashMap::new();
            map.insert(caps["k1"].to_string(), caps["v1"].to_string());
            map.insert(caps["k2"].to_string(), caps["v2"].to_string());
            println!("{:?}", map);
            result.push(Some(map));
        }
        if result.len() != 2 {
            return Err(());
        }
        Ok(ParsedInfo {
            a: result[0].take().unwrap(),
            b: result[1].take().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_regex_content() {
        let test_str = "Key (resource_id, rperiod)=(ocean roon-745, [\"2022-11-04 07:00:00+00\",\"2022-11-08 04:00:00+00\")) conflicts with existing key (resource_id, rperiod)=(ocean roon-745, [\"2022-11-01 07:00:00+00\",\"2022-11-07 04:00:00+00\"))";
        let result: ParsedInfo = test_str.parse().unwrap();
        assert_eq!(
            result.a.get_key_value("resource_id").unwrap(),
            (&"resource_id".to_string(), &"ocean roon-745".to_string())
        );
    }
}
