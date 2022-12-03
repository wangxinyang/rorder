use std::fmt;

use crate::{ReservationStatus, RsvpStatus};

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Unknown => write!(f, "unknown"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// database equivalent of enum status column
impl From<RsvpStatus> for ReservationStatus {
    fn from(status: RsvpStatus) -> Self {
        match status {
            RsvpStatus::Pending => ReservationStatus::Pending,
            RsvpStatus::Confirmed => ReservationStatus::Confirmed,
            RsvpStatus::Blocked => ReservationStatus::Cancelled,
            RsvpStatus::Unknown => ReservationStatus::Unknown,
        }
    }
}
