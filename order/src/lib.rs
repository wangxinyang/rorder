mod error;
mod manager;

use async_trait::async_trait;
use error::ReservationError;
use sqlx::PgPool;

pub type ReservationId = String;

#[async_trait]
pub trait Order {
    /// create reservation
    async fn create_order(
        &self,
        rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError>;

    /// change the status of a reservation(if current status is pending, change it to confirmed)
    async fn change_status(
        &self,
        id: ReservationId,
        status: abi::ReservationStatus,
    ) -> Result<abi::Reservation, ReservationError>;

    /// update_note
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, ReservationError>;

    /// cancel reservation
    async fn cancel_reservation(&self, id: ReservationId) -> Result<(), ReservationError>;

    /// get reservation by id
    async fn get_reservation(
        &self,
        id: ReservationId,
    ) -> Result<abi::Reservation, ReservationError>;

    /// query reservations
    async fn query_reservations(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError>;
}

#[derive(Debug)]
pub struct OrderManager {
    conn: PgPool,
}
