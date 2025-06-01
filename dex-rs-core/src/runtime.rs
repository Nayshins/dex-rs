use std::{future::Future, time::Duration};

pub trait Spawn: Send + Sync + 'static {
    fn spawn(&self, fut: impl Future<Output = ()> + Send + 'static);
}

pub trait Sleep: Send + Sync + 'static {
    type Fut: Future<Output = ()> + Send;
    fn sleep(&self, d: Duration) -> Self::Fut;
}