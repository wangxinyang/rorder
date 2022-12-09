use futures::Stream;
use order::{Order, OrderManager};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::mpsc;
use tonic::{async_trait, Request, Response, Status};

use abi::{
    reservation_service_server::ReservationService, AddRequest, AddResponse, CancelRequest,
    CancelResponse, Config, ConfirmRequest, ConfirmResponse, FilterRequest, FilterResponse,
    GetRequest, GetResponse, ListenRequest, QueryRequest, UpdateRequest, UpdateResponse,
};

use crate::{ReservationResponseStream, TonicReceiverStream};

pub struct RsvpService {
    manager: OrderManager,
}

impl RsvpService {
    pub async fn from_config(config: &Config) -> Result<Self, anyhow::Error> {
        Ok(Self {
            manager: OrderManager::from_config(&config.db).await?,
        })
    }
}

#[async_trait]
impl ReservationService for RsvpService {
    /// make a reservation
    async fn add(&self, request: Request<AddRequest>) -> Result<Response<AddResponse>, Status> {
        let request = request.into_inner();
        if request.reservation.is_none() {
            return Err(Status::invalid_argument("reservation is required"));
        }
        let rsvp = self
            .manager
            .create_order(request.reservation.unwrap())
            .await?;
        Ok(Response::new(AddResponse {
            reservation: Some(rsvp),
        }))
    }

    /// confirm a valid perid resource, if reservation is not pending, do nothing
    async fn confirm(
        &self,
        request: Request<ConfirmRequest>,
    ) -> Result<Response<ConfirmResponse>, Status> {
        let request = request.into_inner();
        if request.id == 0 {
            return Err(Status::invalid_argument("reservation_id is required"));
        }
        let rsvp = self.manager.change_status(request.id).await?;
        Ok(Response::new(ConfirmResponse {
            reservation: Some(rsvp),
        }))
    }

    /// update a reservation
    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let request = request.into_inner();
        let rsvp = self.manager.update_note(request.id, request.note).await?;
        Ok(Response::new(UpdateResponse {
            reservation: Some(rsvp),
        }))
    }

    /// cancel a reservation
    async fn cancel(
        &self,
        request: Request<CancelRequest>,
    ) -> Result<Response<CancelResponse>, Status> {
        let request = request.into_inner();
        let rsvp = self.manager.cancel_reservation(request.id).await?;
        Ok(Response::new(CancelResponse {
            reservation: Some(rsvp),
        }))
    }

    /// get reservation by reservation id
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let request = request.into_inner();
        let rsvp = self.manager.get_reservation(request.id).await?;
        Ok(Response::new(GetResponse {
            reservation: Some(rsvp),
        }))
    }

    type queryStream = ReservationResponseStream;
    /// get reservations by resource id, user id, start time, end time, and status
    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::queryStream>, Status> {
        let request = request.into_inner();
        if request.query.is_none() {
            return Err(Status::invalid_argument("query is required"));
        }
        let rx = self
            .manager
            .query_reservations(request.query.unwrap())
            .await;
        let stream = TonicReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream) as Self::queryStream))
    }

    /// filter reservations, order by reservation id
    async fn filter(
        &self,
        request: Request<FilterRequest>,
    ) -> Result<Response<FilterResponse>, Status> {
        let request = request.into_inner();
        if request.filter.is_none() {
            return Err(Status::invalid_argument("filter is required"));
        }
        let (filter_page, reservations) = self
            .manager
            .filter_reservations(request.filter.unwrap())
            .await?;
        Ok(Response::new(FilterResponse {
            pager: Some(filter_page),
            reservations,
        }))
    }

    type listenStream = ReservationResponseStream;
    /// another system could monitor newly added/confirmed/cancelled reservations
    async fn listen(
        &self,
        _request: Request<ListenRequest>,
    ) -> Result<Response<Self::listenStream>, Status> {
        todo!()
    }
}

impl<T> TonicReceiverStream<T> {
    pub fn new(inner: mpsc::Receiver<Result<T, abi::Error>>) -> Self {
        Self { inner }
    }
}

impl<T> Stream for TonicReceiverStream<T> {
    type Item = Result<T, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_recv(cx) {
            Poll::Ready(Some(Ok(item))) => Poll::Ready(Some(Ok(item))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::test_util::TestConfig;
    use abi::Reservation;

    use super::*;

    #[tokio::test]
    async fn rpc_create_reservation_should_be_work() {
        let config = TestConfig::default();
        let service = RsvpService::from_config(&config).await.unwrap();
        let reservation = Reservation::new_pending(
            "tosei",
            "zoom1",
            "2023-01-25T15:00:00-0700".parse().unwrap(),
            "2023-02-25T12:00:00-0700".parse().unwrap(),
            "test rpc create reservation",
        );
        let request = Request::new(AddRequest {
            reservation: Some(reservation.clone()),
        });
        let response = service.add(request).await.unwrap();
        let reservation1 = response.into_inner().reservation;
        assert!(reservation1.is_some());
        let reservation1 = reservation1.unwrap();
        assert_eq!(reservation1.user_id, reservation.user_id);
        assert_eq!(reservation1.resource_id, reservation.resource_id);
        assert_eq!(reservation1.start_time, reservation.start_time);
        assert_eq!(reservation1.end_time, reservation.end_time);
        assert_eq!(reservation1.note, reservation.note);
        assert_eq!(reservation1.status, reservation.status);
    }
}
