use std::pin::Pin;

use crate::{AsyncError, Database};
use diesel::{
    dsl::Limit,
    query_dsl::{limit_dsl::LimitDsl, load_dsl::ExecuteDsl, LoadQuery},
    result::{Error, OptionalExtension},
    Connection, RunQueryDsl,
};
use futures::Future;

pub trait AsyncRunQueryDsl<Conn>: RunQueryDsl<Conn>
where
    Conn: Connection,
{
    fn execute_async(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<usize, AsyncError<Error>>> + Send + '_>>
    where
        Conn: Connection,
        Self: ExecuteDsl<Conn>;

    fn load_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<U>, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LoadQuery<Conn, U>;

    fn get_result_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<U, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LoadQuery<Conn, U>;

    fn get_optional_result_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<U>, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LoadQuery<Conn, U>;

    fn get_results_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<U>, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LoadQuery<Conn, U>;

    fn first_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<U, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LimitDsl,
        Limit<Self>: LoadQuery<Conn, U>;
}

impl<T: 'static, Conn> AsyncRunQueryDsl<Conn> for T
where
    T: RunQueryDsl<Conn> + Send,
    Conn: Connection,
{
    #[inline]
    fn execute_async(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<usize, AsyncError<Error>>> + Send + '_>>
    where
        Conn: Connection,
        Self: ExecuteDsl<Conn>,
    {
        Box::pin(db.get(move |conn| self.execute(conn)))
    }

    #[inline]
    fn load_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<U>, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LoadQuery<Conn, U>,
    {
        Box::pin(db.get(move |conn| self.load(conn)))
    }

    #[inline]
    fn get_results_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<U>, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LoadQuery<Conn, U>,
    {
        Box::pin(db.get(move |conn| self.get_results(conn)))
    }

    #[inline]
    fn get_result_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<U, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LoadQuery<Conn, U>,
    {
        Box::pin(db.get(move |conn| self.get_result(conn)))
    }

    #[inline]
    fn get_optional_result_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<U>, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LoadQuery<Conn, U>,
    {
        Box::pin(db.get(move |conn| self.get_result(conn).optional()))
    }

    #[inline]
    fn first_async<U: 'static>(
        self,
        db: &Database<Conn>,
    ) -> Pin<Box<dyn Future<Output = Result<U, AsyncError<Error>>> + Send + '_>>
    where
        U: Send,
        Self: LimitDsl,
        Limit<Self>: LoadQuery<Conn, U>,
    {
        Box::pin(db.get(move |conn| self.first(conn)))
    }
}
