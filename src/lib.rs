use actix::Handler;
use actix::MailboxError;
use actix::Message;
use actix::SyncArbiter;
use actix::{Actor, Addr, SyncContext};
use derive_more::{Display, From};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::Connection;
use futures::Future;
use once_cell::sync::OnceCell;
use std::error::Error as StdError;
use std::fmt::{Debug};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

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

pub struct Database<C: 'static>
where
    C: Connection,
{
    cell: Arc<OnceCell<Addr<Executor<C>>>>,
    pool: Pool<ConnectionManager<C>>,
    init: fn(Pool<ConnectionManager<C>>) -> Addr<Executor<C>>,
}

impl<C> Clone for Database<C>
where
    C: Connection,
{
    fn clone(&self) -> Self {
        Database {
            cell: self.cell.clone(),
            init: self.init.clone(),
            pool: self.pool.clone(),
        }
    }
}

impl<C> Database<C>
where
    C: Connection,
{
    #[inline]
    pub fn builder() -> Builder<C> {
        Builder {
            phantom: PhantomData,
            pool_max_size: None,
            pool_min_idle: None,
            pool_max_lifetime: None,
        }
    }

    /// Executes the given function inside a database transaction.
    #[inline]
    pub fn transaction<F, R, E>(&self, f: F) -> impl Future<Item = R, Error = Error>
    where
        F: 'static + FnOnce(&C) -> Result<R, E> + Send,
        R: 'static + Send,
        E: 'static + From<diesel::result::Error> + Debug + Send + Sync,
    {
        self.get(move |conn| conn.transaction(move || f(conn)))
    }

    /// Executes the given function with a connection retrieved from the pool.
    ///
    /// This is non-blocking and uses a `SyncArbiter` to provide a thread pool.
    pub fn get<F, R, E>(&self, f: F) -> impl Future<Item = R, Error = Error>
    where
        F: 'static + FnOnce(&C) -> Result<R, E> + Send,
        R: 'static + Send,
        E: 'static + Debug + Send + Sync,
    {
        self.cell
            .get_or_init(|| (self.init)(self.pool.clone()))
            .send(Execute(f, PhantomData))
            .then(|res| -> Result<R, Error> {
                match res {
                    Ok(res) => match res {
                        Ok(res) => match res {
                            Ok(value) => Ok(value),
                            Err(err) => Err(Error::Execute(Box::new(err))),
                        },

                        Err(err) => Err(err.into()),
                    },

                    Err(err) => Err(err.into()),
                }
            })
    }
}

pub struct Builder<C: 'static>
where
    C: Connection,
{
    phantom: PhantomData<C>,
    pool_max_size: Option<u32>,
    pool_min_idle: Option<Option<u32>>,
    pool_max_lifetime: Option<Option<Duration>>,
}

impl<C> Builder<C>
where
    C: Connection,
{
    #[inline]
    pub fn pool_max_size(&mut self, max_size: u32) -> &mut Self {
        self.pool_max_size = Some(max_size);
        self
    }

    #[inline]
    pub fn pool_min_idle(&mut self, min_idle: Option<u32>) -> &mut Self {
        self.pool_min_idle = Some(min_idle);
        self
    }

    #[inline]
    pub fn pool_max_lifetime(&mut self, max_lifetime: Option<Duration>) -> &mut Self {
        self.pool_max_lifetime = Some(max_lifetime);
        self
    }

    pub fn open(&mut self, url: impl Into<String>) -> Database<C>
    where
        C: Connection,
    {
        let manager = ConnectionManager::<C>::new(url);
        let mut p = Pool::builder();

        if let Some(max_size) = self.pool_max_size {
            p = p.max_size(max_size);
        }

        if let Some(min_idle) = self.pool_min_idle {
            p = p.min_idle(min_idle);
        }

        if let Some(max_lifetime) = self.pool_max_lifetime {
            p = p.max_lifetime(max_lifetime);
        }

        let pool = p.build_unchecked(manager);

        Database {
            pool,
            cell: Arc::new(OnceCell::new()),
            init: |pool| SyncArbiter::start(num_cpus::get(), move || Executor(pool.clone())),
        }
    }
}

#[derive(Debug)]
struct Executor<C: 'static>(Pool<ConnectionManager<C>>)
where
    C: Connection;

impl<C> Actor for Executor<C>
where
    C: Connection,
{
    type Context = SyncContext<Self>;
}

struct Execute<F, C, R, E>(F, PhantomData<(C, R)>)
where
    R: 'static + Send,
    E: 'static + Debug + Send + Sync,
    C: Connection,
    F: FnOnce(&C) -> Result<R, E>;

impl<F, C, R, E> Message for Execute<F, C, R, E>
where
    R: Send,
    E: Debug + Send + Sync,
    C: Connection,
    F: FnOnce(&C) -> Result<R, E>,
{
    type Result = Result<Result<R, E>, r2d2::Error>;
}

impl<F, C, R, E> Handler<Execute<F, C, R, E>> for Executor<C>
where
    R: Send,
    E: Debug + Send + Sync,
    C: Connection,
    F: FnOnce(&C) -> Result<R, E>,
{
    type Result = Result<Result<R, E>, r2d2::Error>;

    fn handle(&mut self, msg: Execute<F, C, R, E>, _: &mut Self::Context) -> Self::Result {
        let conn = match self.0.get() {
            Ok(conn) => conn,
            Err(err) => return Err(err),
        };

        Ok((msg.0)(&*conn))
    }
}
