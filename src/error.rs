use derive_more::Display;
use std::{error::Error as StdError, fmt::Debug};

#[derive(Debug, Display)]
pub enum AsyncError<E>
where
    E: 'static + Debug + Send,
{
    // Error occured when attempting to deliver the query to the Database actor
    #[display(fmt = "{}", _0)]
    Delivery(actix::MailboxError),

    // Timed out trying to checkout a connection
    #[display(fmt = "{}", _0)]
    Timeout(r2d2::Error),

    // An error occurred when interacting with the database
    #[display(fmt = "{:?}", _0)]
    Execute(E),
}

impl<E> StdError for AsyncError<E> where E: 'static + Debug + Send {}

#[cfg(feature = "actix-web")]
impl<E> actix_web::ResponseError for AsyncError<E> where E: 'static + Debug + Send {}

#[cfg(feature = "failure")]
impl AsyncError<failure::Error> {
    /// Attempts to downcast this Error to a particular Fail type.
    #[inline]
    pub fn downcast<T: failure::Fail>(self) -> Result<T, Self> {
        match self {
            AsyncError::Execute(err) => err.downcast().map_err(AsyncError::Execute),
            err => Err(err),
        }
    }

    /// Attempts to downcast this Error to a particular Fail type by reference.
    #[inline]
    pub fn downcast_ref<T: failure::Fail>(&self) -> Option<&T> {
        match self {
            AsyncError::Execute(err) => err.downcast_ref(),
            _ => None,
        }
    }

    /// Attempts to downcast this Error to a particular Fail type by mutable reference.
    #[inline]
    pub fn downcast_mut<T: failure::Fail>(&mut self) -> Option<&mut T> {
        match self {
            AsyncError::Execute(err) => err.downcast_mut(),
            _ => None,
        }
    }
}
