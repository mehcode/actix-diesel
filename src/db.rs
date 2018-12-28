use crate::{
    executor::{Execute, Executor},
    Builder, Error,
};
use actix::Addr;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection,
};
use futures::Future;
use once_cell::sync::OnceCell;
use std::{fmt::Debug, marker::PhantomData, sync::Arc};

pub struct Database<C: 'static>
where
    C: Connection,
{
    pub(crate) cell: Arc<OnceCell<Addr<Executor<C>>>>,
    pub(crate) pool: Pool<ConnectionManager<C>>,
    pub(crate) init: fn(Pool<ConnectionManager<C>>) -> Addr<Executor<C>>,
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
    pub fn open(url: impl Into<String>) -> Database<C> {
        Self::builder().open(url)
    }

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
