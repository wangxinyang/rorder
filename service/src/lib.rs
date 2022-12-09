mod server;
mod test_util;

use abi::Reservation;
use futures::Stream;
use std::pin::Pin;
use tonic::Status;

pub use server::*;
pub use test_util::*;

type ReservationResponseStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;
