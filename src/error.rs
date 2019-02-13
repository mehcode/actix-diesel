use derive_more::Display;
use std::error::Error as StdError;

#[derive(Debug, Display)]
pub enum AsyncError {
    // Error occured when attempting to deliver the query to the Database actor
    #[display(fmt = "{}", _0)]
    Delivery(actix::MailboxError),

    // Timed out trying to checkout a connection
    #[display(fmt = "{}", _0)]
    Timeout(r2d2::Error),

    // An error occurred when interacting with the database
    #[display(fmt = "{:?}", _0)]
    Query(diesel::result::Error),
}

impl StdError for AsyncError {}

#[cfg(feature = "actix-web")]
impl actix_web::ResponseError for AsyncError {}
