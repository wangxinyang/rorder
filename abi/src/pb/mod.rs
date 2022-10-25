// solve the generate code [you are deriving `PartialEq` and can implement `Eq`] problem by clippy::all
// solve the camel problem by non_camel_case_types
#[allow(clippy::all, non_camel_case_types)]
mod rsvp;

pub use rsvp::*;
