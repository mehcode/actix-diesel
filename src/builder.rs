use crate::{executor::Executor, Database};
use actix::SyncArbiter;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection,
};
use once_cell::sync::OnceCell;
use std::{marker::PhantomData, sync::Arc, time::Duration};

pub struct Builder<C: 'static>
where
    C: Connection,
{
    pub(crate) phantom: PhantomData<C>,
    pub(crate) pool_max_size: Option<u32>,
    pub(crate) pool_min_idle: Option<Option<u32>>,
    pub(crate) pool_max_lifetime: Option<Option<Duration>>,
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
