use crate::{Order, OrderManager, ReservationId};
use abi::{convert_to_utc_time, Error, ReservationQuery, ReservationStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, Row};

#[async_trait]
impl Order for OrderManager {
    async fn create_order(
        &self,
        mut rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, sqlx::Error> {
        // rsvp.validate()?;

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
    ) -> Result<abi::Reservation, Error> {
        todo!()
    }

    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, Error> {
        todo!()
    }

    async fn cancel_reservation(&self, _id: ReservationId) -> Result<(), Error> {
        todo!()
    }

    async fn get_reservation(&self, _id: ReservationId) -> Result<abi::Reservation, Error> {
        todo!()
    }

    async fn query_reservations(
        &self,
        _query: ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use abi::{Reservation, ReservationConflict, ReservationConflictInfo, ReservationWindow};
    use chrono::FixedOffset;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reservation_should_be_work() {
        let order_manage = OrderManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-11-01T15:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-11-07T12:00:00+0800".parse().unwrap();
        let rsvp = Reservation::new_pending(
            "tosei",
            "ocean roon-745",
            start,
            end,
            "please check the room for me",
        );

        let rsvp = order_manage.create_order(rsvp).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reservation_should_be_conflict() {
        let order_manage = OrderManager::new(migrated_pool.clone());

        let rsvp1 = Reservation::new_pending(
            "tosei",
            "ocean roon-745",
            "2022-11-01T15:00:00+0800".parse().unwrap(),
            "2022-11-07T12:00:00+0800".parse().unwrap(),
            "please check the room for me",
        );

        let rsvp2 = Reservation::new_pending(
            "wxy",
            "ocean roon-745",
            "2022-11-04T15:00:00+0800".parse().unwrap(),
            "2022-11-08T12:00:00+0800".parse().unwrap(),
            "love this room",
        );

        let _rsvp1 = order_manage.create_order(rsvp1).await.unwrap();
        let error_rsvp2 = order_manage.create_order(rsvp2).await.unwrap_err();
        println!("----{:?}", error_rsvp2);
        let _info = ReservationConflictInfo::Parsed(ReservationConflict {
            a: ReservationWindow {
                rid: "ocean-view-room-713".to_string(),
                start: "2022-12-26T15:00:00-0700".parse().unwrap(),
                end: "2022-12-30T12:00:00-0700".parse().unwrap(),
            },
            b: ReservationWindow {
                rid: "ocean-view-room-713".to_string(),
                start: "2022-12-25T15:00:00-0700".parse().unwrap(),
                end: "2022-12-28T12:00:00-0700".parse().unwrap(),
            },
        });
        // assert_eq!(error_rsvp2, abi::Error::ConfilictReservation(info));
    }
}
