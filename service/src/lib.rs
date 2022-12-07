mod server;

use abi::Reservation;
use futures::Stream;
use std::pin::Pin;
use tonic::Status;

pub use server::*;

type ReservationResponseStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;
