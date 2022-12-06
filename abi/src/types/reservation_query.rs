use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;
use std::ops::Bound;

use crate::{convert_to_utc_time, Error, ReservationQuery, Validator};

// #[allow(clippy::too_many_arguments)] use the derive_builder solve this clippy problem
impl ReservationQuery {
    pub fn timespan(&self) -> PgRange<DateTime<Utc>> {
        PgRange {
            start: Bound::Included(convert_to_utc_time(self.start.as_ref().unwrap())),
            end: Bound::Excluded(convert_to_utc_time(self.end.as_ref().unwrap())),
        }
    }
}

impl Validator for ReservationQuery {
    fn validate(&self) -> Result<(), crate::Error> {
        if self.start.is_none() || self.end.is_none() {
            return Err(Error::InvalidTime);
        }

        let start = self.start.as_ref().unwrap();
        let end = self.end.as_ref().unwrap();
        if start.seconds >= end.seconds {
            return Err(Error::InvalidTime);
        }

        Ok(())
    }
}
