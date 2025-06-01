use std::{future::Future, time::Duration};

use crate::runtime::{Sleep, Spawn};
use tokio::time::Sleep as TokioSleep;

#[derive(Clone, Copy, Debug, Default)]
pub struct TokioRt;

impl Spawn for TokioRt {
    fn spawn(&self, fut: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(fut);
    }
}

impl Sleep for TokioRt {
    type Fut = TokioSleep;

    fn sleep(&self, d: Duration) -> Self::Fut {
        tokio::time::sleep(d)
    }
}
