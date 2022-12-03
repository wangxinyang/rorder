mod error;
mod pb;
mod types;
mod utils;

pub use error::*;
pub use pb::*;
pub use types::*;
pub use utils::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
enum RsvpStatus {
    Pending,
    Confirmed,
    Blocked,
    Unknown,
}
