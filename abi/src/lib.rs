mod pb;

use chrono::{DateTime, NaiveDateTime, Utc};
pub use pb::*;
use prost_types::Timestamp;

/// convert timestamp to chrono datetime
pub fn convert_to_utc_time(time: &Timestamp) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(time.seconds, time.nanos as _),
        Utc,
    )
}
