use std::fmt;

use crate::ReservationStatus;

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
