use actix::MailboxError;
use derive_more::{Display, From};
use std::{error::Error as StdError, fmt::Debug};

#[derive(Debug, Display, From)]
pub enum Error {
    // Occured during message deliver to sync actor
    #[display(fmt = "{}", _0)]
    Delivery(MailboxError),

    // Occured during connection checkout from pool
    #[display(fmt = "{}", _0)]
    Checkout(r2d2::Error),

    // Occured during execute
    #[display(fmt = "{:?}", _0)]
    Execute(Box<dyn Debug + Send + Sync>),
}

impl StdError for Error {}

#[cfg(feature = "actix-web")]
impl actix_web::ResponseError for Error {}
