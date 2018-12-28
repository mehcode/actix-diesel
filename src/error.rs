use actix::MailboxError;
use derive_more::Display;
use std::{error::Error as StdError, fmt::Debug};

#[derive(Debug, Display)]
pub enum Error<E>
where
    E: 'static + Debug + Send + Sync,
{
    // Occured during message delivery to sync actor
    #[display(fmt = "{}", _0)]
    Delivery(MailboxError),

    // Occured during connection checkout from pool
    #[display(fmt = "{}", _0)]
    Checkout(r2d2::Error),

    // Occured during execute
    #[display(fmt = "{:?}", _0)]
    Execute(E),
}

impl<E> StdError for Error<E> where E: 'static + Debug + Send + Sync {}

#[cfg(feature = "actix-web")]
impl<E> actix_web::ResponseError for Error<E> where E: 'static + Debug + Send + Sync {}
