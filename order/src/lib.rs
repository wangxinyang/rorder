mod manager;

use abi::{Error, FilterPager};
use async_trait::async_trait;
use sqlx::PgPool;

pub type ReservationId = i64;

#[async_trait]
pub trait Order {
    /// create reservation
    async fn create_order(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, Error>;

    /// change the status of a reservation(if current status is pending, change it to confirmed)
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, Error>;

    /// update_note
    async fn update_note(&self, id: ReservationId, note: String)
        -> Result<abi::Reservation, Error>;

    /// cancel reservation
    async fn cancel_reservation(&self, id: ReservationId) -> Result<(), Error>;

    /// get reservation by id
    async fn get_reservation(&self, id: ReservationId) -> Result<abi::Reservation, Error>;

    /// query reservations
    async fn query_reservations(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, Error>;

    /// query reservations by order by id
    async fn filter_reservations(
        &self,
        filter: abi::ReservationFilter,
    ) -> Result<(FilterPager, Vec<abi::Reservation>), Error>;
}

#[derive(Debug)]
pub struct OrderManager {
    conn: PgPool,
}

impl OrderManager {
    pub fn new(conn: PgPool) -> Self {
        Self { conn }
    }
}
