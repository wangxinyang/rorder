use crate::{error::ReservationError, Order, OrderManager, ReservationId};
use abi::{convert_to_utc_time, ReservationQuery, ReservationStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, Row};

#[async_trait]
impl Order for OrderManager {
    async fn create_order(
        &self,
        mut rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError> {
        if rsvp.start_time.is_none() || rsvp.end_time.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        // can not get Timestamp of prost_type, because not import tonic crate
        let start = convert_to_utc_time(rsvp.start_time.as_ref().unwrap());
        let end = convert_to_utc_time(rsvp.end_time.as_ref().unwrap());
        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let id: i64 = sqlx::query(
            "INSERT INTO reservation (user_id, resource_id, timespan, status, note)
            VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.status)
        .bind(rsvp.note.clone())
        .fetch_one(&self.conn)
        .await?
        .get(0);

        rsvp.id = id;
        Ok(rsvp)
    }

    async fn change_status(
        &self,
        _id: ReservationId,
        _status: ReservationStatus,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn cancel_reservation(&self, _id: ReservationId) -> Result<(), ReservationError> {
        todo!()
    }

    async fn get_reservation(
        &self,
        _id: ReservationId,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn query_reservations(
        &self,
        _query: ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        todo!()
    }
}
