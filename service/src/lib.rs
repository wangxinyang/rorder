use futures::Stream;
use std::pin::Pin;
use tonic::{Request, Response, Status};

use abi::{
    reservation_service_server::ReservationService, AddRequest, AddResponse, CancelRequest,
    CancelResponse, ConfirmRequest, ConfirmResponse, FilterRequest, FilterResponse, GetRequest,
    GetResponse, ListenRequest, QueryRequest, Reservation, UpdateRequest, UpdateResponse,
};

type ReservationResponseStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;
pub struct RsvpService;

#[tonic::async_trait]
impl ReservationService for RsvpService {
    /// make a reservation
    async fn add(&self, _request: Request<AddRequest>) -> Result<Response<AddResponse>, Status> {
        todo!()
    }

    /// confirm a valid perid resource, if reservation is not pending, do nothing
    async fn confirm(
        &self,
        _request: Request<ConfirmRequest>,
    ) -> Result<Response<ConfirmResponse>, Status> {
        todo!()
    }

    /// update a reservation
    async fn update(
        &self,
        _request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        todo!()
    }

    /// cancel a reservation
    async fn cancel(
        &self,
        _request: Request<CancelRequest>,
    ) -> Result<Response<CancelResponse>, Status> {
        todo!()
    }

    /// get reservation by reservation id
    async fn get(&self, _request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        todo!()
    }

    type queryStream = ReservationResponseStream;
    /// get reservations by resource id, user id, start time, end time, and status
    async fn query(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<Response<Self::queryStream>, Status> {
        todo!()
    }

    /// filter reservations, order by reservation id
    async fn filter(
        &self,
        _request: Request<FilterRequest>,
    ) -> Result<Response<FilterResponse>, Status> {
        todo!()
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
