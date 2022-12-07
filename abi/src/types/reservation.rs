use chrono::{DateTime, FixedOffset, Utc};
use sqlx::{
    postgres::{types::PgRange, PgRow},
    FromRow, Row,
};
use std::ops::Bound;

use crate::{convert_to_timestamp, Error, Reservation, ReservationStatus, RsvpStatus, Validator};

impl Reservation {
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: 0,
            user_id: uid.into(),
            resource_id: rid.into(),
            start_time: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end_time: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
            status: ReservationStatus::Pending as i32,
        }
    }
}

impl Validator for Reservation {
    fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }

        if self.start_time.is_none() || self.end_time.is_none() {
            return Err(Error::InvalidTime);
        }

        let start = self.start_time.as_ref().unwrap();
        let end = self.end_time.as_ref().unwrap();
        if start.seconds >= end.seconds {
            return Err(Error::InvalidTime);
        }

        Ok(())
    }
}

/// implement the FromRow trait for Reservation query_as macro
impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id: i64 = row.get("id");

        let period: PgRange<DateTime<Utc>> = row.get("rperiod");
        let period: NaviRange<DateTime<Utc>> = period.into();
        let start = convert_to_timestamp(period.start.unwrap());
        let end = convert_to_timestamp(period.end.unwrap());

        let status: RsvpStatus = row.get("rstatus");

        Ok(Self {
            id,
            user_id: row.get("user_id"),
            resource_id: row.get("resource_id"),
            start_time: Some(start),
            end_time: Some(end),
            status: ReservationStatus::from(status) as i32,
            note: row.get("note"),
        })
    }
}

struct NaviRange<T> {
    start: Option<T>,
    end: Option<T>,
}

impl<T> From<PgRange<T>> for NaviRange<T> {
    fn from(range: PgRange<T>) -> Self {
        let f = |b: Bound<T>| match b {
            Bound::Excluded(v) => Some(v),
            Bound::Included(v) => Some(v),
            Bound::Unbounded => None,
        };
        let start = f(range.start);
        let end = f(range.end);
        Self { start, end }
    }
}
