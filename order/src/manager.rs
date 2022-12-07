use crate::{Order, OrderManager, ReservationId};
use abi::{
    convert_to_utc_time, DbConfig, Error, FilterPager, ReservationQuery, ReservationStatus,
    Validator,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{
    postgres::{types::PgRange, PgPoolOptions},
    PgPool, Row,
};

impl OrderManager {
    pub fn new(conn: PgPool) -> Self {
        Self { conn }
    }

    pub async fn from_config(config: &DbConfig) -> Result<Self, Error> {
        let url = config.url();
        let conn = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&url)
            .await?;
        Ok(Self::new(conn))
    }
}

#[async_trait]
impl Order for OrderManager {
    async fn create_order(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, Error> {
        rsvp.validate()?;

        let status = ReservationStatus::from_i32(rsvp.status).unwrap_or(ReservationStatus::Pending);

        // can not get Timestamp of prost_type, because not import tonic crate
        let start = convert_to_utc_time(rsvp.start_time.as_ref().unwrap());
        let end = convert_to_utc_time(rsvp.end_time.as_ref().unwrap());
        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let id: i64 = sqlx::query(
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

        rsvp.id = id;
        Ok(rsvp)
    }

    /// update the status of reservation resource by id
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        // 不使用sqlx::query_as!的原因是struct的字段和数据库字段名字不匹配,无法解析出来
        // let reservation  = sqlx::query_as!(Reservation,
        //     "update rsvt.reservations set rstatus = 'confirmed' where id = $1 and rstatus = 'pending' RETURNING *",
        //     id
        // )
        // .fetch_one(&self.conn)
        // .await?;
        let reservation = sqlx::query_as(
            "update rsvt.reservations set rstatus = 'confirmed' where id = $1 and rstatus = 'pending' RETURNING *"
        )
        .bind(id)
        .fetch_one(&self.conn)
        .await?;
        Ok(reservation)
    }

    /// modify the reservation note info
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, Error> {
        let rsvp =
            sqlx::query_as("update rsvt.reservations set note = $1 where id = $2 RETURNING *")
                .bind(note)
                .bind(id)
                .fetch_one(&self.conn)
                .await?;
        Ok(rsvp)
    }

    /// cancel the book reservation resource
    async fn cancel_reservation(&self, id: ReservationId) -> Result<(), Error> {
        sqlx::query("update rsvt.reservations set rstatus = 'pending' where id = $1 RETURNING *")
            .bind(id)
            .fetch_one(&self.conn)
            .await?;
        Ok(())
    }

    /// get reservation resources by id
    async fn get_reservation(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        let rsvp = sqlx::query_as("select * from rsvt.reservations where id = $1")
            .bind(id)
            .fetch_one(&self.conn)
            .await?;
        Ok(rsvp)
    }

    /// call postgreSql function get reservation resources
    async fn query_reservations(
        &self,
        query: ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, Error> {
        let user_id = str_to_option(&query.user_id);
        let resource_id = str_to_option(&query.resource_id);
        let range = query.timespan();
        let status =
            ReservationStatus::from_i32(query.status).unwrap_or(ReservationStatus::Pending);
        let rsvps = sqlx::query_as(
            "select * from rsvt.query($1, $2, $3, $4::rsvt.reservation_status, $5, $6, $7)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(range)
        .bind(status.to_string())
        .bind(query.page)
        .bind(query.desc)
        .bind(query.page_size)
        .fetch_all(&self.conn)
        .await?;

        Ok(rsvps)
    }

    async fn filter_reservations(
        &self,
        filter: abi::ReservationFilter,
    ) -> Result<(FilterPager, Vec<abi::Reservation>), Error> {
        let user_id = str_to_option(&filter.user_id);
        let resource_id = str_to_option(&filter.resource_id);
        let status =
            ReservationStatus::from_i32(filter.status).unwrap_or(ReservationStatus::Pending);
        let rsvps: Vec<abi::Reservation> = sqlx::query_as(
            "select * from rsvt.filter($1, $2, $3::rsvt.reservation_status, $4, $5, $6)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(status.to_string())
        .bind(filter.cursor)
        .bind(filter.desc)
        .bind(filter.page_size)
        .fetch_all(&self.conn)
        .await?;

        let pager = FilterPager {
            prev: Some(rsvps[0].id),
            next: Some(rsvps[rsvps.len() - 1].id),
            // TODO: how to get total count?
            total: Some(0),
        };

        Ok((pager, rsvps))
    }
}

fn str_to_option(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use abi::{
        Reservation, ReservationConflict, ReservationConflictInfo, ReservationFilterBuilder,
        ReservationQueryBuilder, ReservationWindow,
    };
    use chrono::FixedOffset;
    use prost_types::Timestamp;
    use sqlx::PgPool;

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
        assert!(rsvp.id != 0);
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
        let error_rsvp2: abi::Error = order_manage.create_order(rsvp2).await.unwrap_err();
        let info = ReservationConflictInfo::Parsed(ReservationConflict {
            new: ReservationWindow {
                rid: "ocean roon-745".to_string(),
                start: "2022-11-04T15:00:00+0800".parse().unwrap(),
                end: "2022-11-08T12:00:00+0800".parse().unwrap(),
            },
            old: ReservationWindow {
                rid: "ocean roon-745".to_string(),
                start: "2022-11-01T15:00:00+0800".parse().unwrap(),
                end: "2022-11-07T12:00:00+0800".parse().unwrap(),
            },
        });
        assert_eq!(error_rsvp2, abi::Error::ConfilictReservation(info));
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_reservation_status_should_be_work() {
        let order_manage = OrderManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-12-03T15:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-12-11T12:00:00+0800".parse().unwrap();
        let rsvp = Reservation::new_pending("tosei", "room-test-1", start, end, "book room");
        let rsvp = order_manage.create_order(rsvp).await.unwrap();
        let rsvp = order_manage.change_status(rsvp.id).await.unwrap();
        assert_eq!(
            ReservationStatus::Confirmed,
            ReservationStatus::from_i32(rsvp.status).unwrap()
        );
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_reservation_status_twice_should_not_be_work() {
        let order_manage = OrderManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-12-03T15:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-12-11T12:00:00+0800".parse().unwrap();
        let rsvp = Reservation::new_pending("tosei", "room-test-1", start, end, "book room");
        let rsvp = order_manage.create_order(rsvp).await.unwrap();
        let rsvp = order_manage.change_status(rsvp.id).await.unwrap();
        // update the status twice occurs the RowNotFound error
        let rsvp = order_manage.change_status(rsvp.id).await.unwrap_err();
        assert_eq!(Error::NotFound, rsvp);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_reservation_note_should_be_work() {
        let order_manage = OrderManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-12-03T15:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-12-11T12:00:00+0800".parse().unwrap();
        let rsvp = Reservation::new_pending(
            "tosei",
            "room-test-1",
            start,
            end,
            "i love this room very much, please book this room for me",
        );
        let rsvp = order_manage.create_order(rsvp).await.unwrap();
        let rsvp = order_manage
            .update_note(rsvp.id, "please cancel this room".into())
            .await
            .unwrap();
        assert_eq!("please cancel this room".to_string(), rsvp.note);
        assert_ne!(
            "i love this room very much, please book this room for me".to_string(),
            rsvp.note
        )
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn cancel_reservation_should_be_work() {
        let order_manage = OrderManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-12-03T15:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-12-11T12:00:00+0800".parse().unwrap();
        let rsvp = Reservation::new_pending(
            "tosei",
            "room-test-1",
            start,
            end,
            "i love this room very much, please book this room for me",
        );
        // create the reservation
        let rsvp = order_manage.create_order(rsvp).await.unwrap();
        order_manage.cancel_reservation(rsvp.id).await.unwrap();
        let get_rsvp_info = order_manage.get_reservation(rsvp.id).await.unwrap();
        assert_eq!(get_rsvp_info.status, rsvp.status);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn get_reservation_should_be_work() {
        let order_manage = OrderManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-12-03T15:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-12-11T12:00:00+0800".parse().unwrap();
        let rsvp = Reservation::new_pending(
            "tosei",
            "room-test-1",
            start,
            end,
            "i love this room very much, please book this room for me",
        );
        // create the reservation
        let rsvp = order_manage.create_order(rsvp).await.unwrap();
        let get_rsvp_info = order_manage.get_reservation(rsvp.id).await.unwrap();
        assert_eq!(get_rsvp_info, rsvp);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn query_reservations_should_be_work() {
        // let order_manage = OrderManager::new(migrated_pool.clone());
        let (rsvp, manager) = make_alice_reservation(migrated_pool.clone()).await;
        // this is not database data, created by make_alice_reservation method
        // use the builder pattern for warp the ReservationQuery struct
        let query = ReservationQueryBuilder::default()
            .user_id("aliceid")
            .status(ReservationStatus::Pending)
            .start("2021-10-01T15:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2023-12-30T15:00:00-0700".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();
        // query the reservation
        let rsvps = manager.query_reservations(query).await.unwrap();
        assert_eq!(1, rsvps.len());
        assert_eq!(rsvp, rsvps[0]);

        // if window is not blank
        let query = ReservationQueryBuilder::default()
            .user_id("aliceid")
            .status(ReservationStatus::Confirmed)
            .start("2023-01-01T15:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2023-02-01T15:00:00-0700".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();
        // query the reservation
        let rsvps = manager.query_reservations(query).await.unwrap();
        assert_eq!(0, rsvps.len());

        // if status is not in correct
        let query = ReservationQueryBuilder::default()
            .user_id("aliceid")
            .status(ReservationStatus::Confirmed)
            .start("2021-10-01T15:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2023-12-30T15:00:00-0700".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();
        // query the reservation
        let rsvps = manager.query_reservations(query.clone()).await.unwrap();
        assert_eq!(0, rsvps.len());

        // if change the status to confirmed, query should get result
        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        let rsvps = manager.query_reservations(query.clone()).await.unwrap();
        assert_eq!(1, rsvps.len());
        assert_eq!(rsvp, rsvps[0]);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn filter_reservations_should_be_work() {
        let (rsvp, manager) = make_alice_reservation(migrated_pool.clone()).await;
        // this is not database data, created by make_alice_reservation method
        // use the builder pattern for warp the ReservationQuery struct
        let filter = ReservationFilterBuilder::default()
            .status(ReservationStatus::Pending)
            .desc(true)
            .build()
            .unwrap();
        // query the reservation
        let (filter_page, rsvps) = manager.filter_reservations(filter).await.unwrap();
        assert_eq!(1, rsvps.len());
        assert_eq!(rsvp, rsvps[0]);
        assert_eq!(1, filter_page.prev.unwrap());
        assert_eq!(1, filter_page.next.unwrap());
    }

    async fn make_alice_reservation(pool: PgPool) -> (Reservation, OrderManager) {
        make_reservation(
            pool,
            "aliceid",
            "ixia-test-1",
            "2023-01-25T15:00:00-0700",
            "2023-02-25T12:00:00-0700",
            "I need to book this for xyz project for a month.",
        )
        .await
    }

    async fn make_reservation(
        pool: PgPool,
        uid: &str,
        rid: &str,
        start: &str,
        end: &str,
        note: &str,
    ) -> (Reservation, OrderManager) {
        let manager = OrderManager::new(pool.clone());
        let rsvp = abi::Reservation::new_pending(
            uid,
            rid,
            start.parse().unwrap(),
            end.parse().unwrap(),
            note,
        );

        (manager.create_order(rsvp).await.unwrap(), manager)
    }
}
