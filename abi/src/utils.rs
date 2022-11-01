use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use prost_types::Timestamp;

/// convert timestamp to chrono datetime
pub fn convert_to_utc_time(time: &Timestamp) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(time.seconds, time.nanos as _),
        Utc,
    )
}

pub fn convert_to_timestamp(time: DateTime<FixedOffset>) -> Timestamp {
    Timestamp {
        seconds: time.timestamp(),
        nanos: time.timestamp_subsec_nanos() as _,
    }
}
