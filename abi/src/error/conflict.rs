use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReservationConflict {
    pub a: ReservationWindow,
    pub b: ReservationWindow,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedInfo::from_str(s)?.try_into()
    }
}

#[derive(Debug)]
struct ParsedInfo {
    a: HashMap<String, String>,
    b: HashMap<String, String>,
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();

    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            a: value.a.try_into()?,
            b: value.b.try_into()?,
        })
    }
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = ();

    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let rperoid_str = value.get("rperiod").ok_or(())?.replace(['"', '['], "");
        let mut period = rperoid_str.splitn(2, ',');
        let start = parse_datetime(period.next().ok_or(())?.trim())?;
        let end = parse_datetime(period.next().ok_or(())?.trim())?;
        Ok(Self {
            rid: value.get("resource_id").ok_or(())?.to_string(),
            start,
            end,
        })
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|e| println!("err is: {}", e))?
        .into())
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

    const ERR_MSG: &str = "Key (resource_id, rperiod)=(ocean roon-745, [\"2022-11-04 07:00:00+00\",
    \"2022-11-08 04:00:00+00\")) conflicts with existing key (resource_id, rperiod)=(ocean roon-745,
        [\"2022-11-01 07:00:00+00\",\"2022-11-07 04:00:00+00\"))";

    #[test]
    fn parse_datatime_should_work() {
        let dt = parse_datetime("2022-11-04 07:00:00+00").unwrap();
        assert_eq!(dt.to_rfc3339(), "2022-11-04T07:00:00+00:00");
    }

    #[test]
    fn get_regex_content() {
        let result: ParsedInfo = ERR_MSG.parse().unwrap();
        assert_eq!(
            result.a.get_key_value("resource_id").unwrap(),
            (&"resource_id".to_string(), &"ocean roon-745".to_string())
        );
    }

    #[test]
    fn hash_map_to_reservation_window_should_work() {
        let mut map = HashMap::new();
        map.insert("resource_id".to_string(), "ocean roon-745".to_string());
        map.insert(
            "rperiod".to_string(),
            "[\"2022-11-04 07:00:00+00\",\"2022-11-08 04:00:00+00\"".to_string(),
        );
        let reservation_window: ReservationWindow = map.try_into().unwrap();
        assert_eq!("ocean roon-745", reservation_window.rid);
        let start: DateTime<Utc> = parse_datetime("2022-11-04 07:00:00+00").unwrap();
        assert_eq!(start, reservation_window.start);
        let end: DateTime<Utc> = parse_datetime("2022-11-08 04:00:00+00").unwrap();
        assert_eq!(end, reservation_window.end);
    }

    #[test]
    fn conflict_error_message_should_parse() {
        let result: ParsedInfo = ERR_MSG.parse().unwrap();
        println!("result is : {:?}", result);
        let reservation_conflict: ReservationConflict = result.try_into().unwrap();
        assert_eq!("ocean roon-745", reservation_conflict.a.rid);
        assert_eq!(
            parse_datetime("2022-11-04 07:00:00+00").unwrap(),
            reservation_conflict.a.start
        );
        assert_eq!(
            parse_datetime("2022-11-08 04:00:00+00").unwrap(),
            reservation_conflict.a.end
        );
        assert_eq!("ocean roon-745", reservation_conflict.b.rid);
        assert_eq!(
            parse_datetime("2022-11-01 07:00:00+00").unwrap(),
            reservation_conflict.b.start
        );
        assert_eq!(
            parse_datetime("2022-11-07 04:00:00+00").unwrap(),
            reservation_conflict.b.end
        );
    }
}
