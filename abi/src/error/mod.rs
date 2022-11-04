mod conflict;

pub use conflict::*;

use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error")]
    DbError(sqlx::Error),

    #[error("Conflict Reservation")]
    ConfilictReservation(ReservationConflictInfo),

    #[error("Invalid start or end time for the reservation")]
    InvalidTime,

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("unknown error")]
    Unknown,
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvt"), Some("reservations")) => {
                        Error::ConfilictReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            _ => Error::DbError(e),
        }
    }
}
