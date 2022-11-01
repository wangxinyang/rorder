use chrono::{DateTime, FixedOffset};

use crate::{convert_to_timestamp, convert_to_utc_time, Error, Reservation, ReservationStatus};

impl Reservation {
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id: uid.into(),
            resource_id: rid.into(),
            start_time: Some(convert_to_timestamp(start)),
            end_time: Some(convert_to_timestamp(end)),
            note: note.into(),
            status: ReservationStatus::Pending as i32,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }

        if self.start_time.is_none() || self.end_time.is_none() {
            return Err(Error::InvalidTime);
        }

        let start = convert_to_utc_time(self.start_time.as_ref().unwrap());
        let end = convert_to_utc_time(self.end_time.as_ref().unwrap());
        if start >= end {
            return Err(Error::InvalidTime);
        }

        Ok(())
    }
}
