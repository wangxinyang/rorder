use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;
use sqlx::types::Uuid;

use crate::Error;

/// convert timestamp to chrono datetime
pub fn convert_to_utc_time(time: &Timestamp) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        // Deprecated since 0.4.23
        // use of deprecated associated function `chrono::NaiveDateTime::from_timestamp`
        // : use `from_timestamp_opt()` instead
        NaiveDateTime::from_timestamp_opt(time.seconds, time.nanos as _).unwrap(),
        Utc,
    )
}

/// convert DateTime<Utc> to prost_types timestamp
pub fn convert_to_timestamp(time: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: time.timestamp(),
        nanos: time.timestamp_subsec_nanos() as _,
    }
}

/// convert String id to sqlx Uuid
pub fn get_uuid_from_string(id: String) -> Result<Uuid, Error> {
    id.as_str()
        .try_into()
        .map_err(|_| Error::InvalidReservationId(id.clone()))
}
