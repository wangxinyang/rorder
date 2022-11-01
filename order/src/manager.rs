use crate::{error::ReservationError, Order, OrderManager, ReservationId};
use abi::{convert_to_utc_time, ReservationQuery, ReservationStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, Row};

#[async_trait]
impl Order for OrderManager {
    async fn create_order(
        &self,
        mut rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError> {
        if rsvp.start_time.is_none() || rsvp.end_time.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let status = ReservationStatus::from_i32(rsvp.status).unwrap_or(ReservationStatus::Pending);

        // can not get Timestamp of prost_type, because not import tonic crate
        let start = convert_to_utc_time(rsvp.start_time.as_ref().unwrap());
        let end = convert_to_utc_time(rsvp.end_time.as_ref().unwrap());
        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let id: Uuid = sqlx::query(
            "INSERT INTO rsvt.reservations (user_id, resource_id, rperiod, rstatus, note)
            VALUES ($1, $2, $3, $4::rsvt.reservation_status, $5) RETURNING id",
        )
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(status.to_string())
        .bind(rsvp.note.clone())
        .fetch_one(&self.conn)
        .await?
        .get(0);

        rsvp.id = id.to_string();
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

#[cfg(test)]
mod tests {
    use abi::convert_to_timestamp;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reservation_should_be_work() {
        let order_manage = OrderManager::new(migrated_pool.clone());

        let rsvp = abi::Reservation {
            id: "".to_string(),
            user_id: "tosei".to_string(),
            resource_id: "ocean room-745".to_string(),
            start_time: Some(convert_to_timestamp(Utc::now())),
            end_time: Some(convert_to_timestamp(Utc::now())),
            status: ReservationStatus::Pending as i32,
            note: "note".to_string(),
        };

        let rsvp = order_manage.create_order(rsvp).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }
}
