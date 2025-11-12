//! Middleware for request/response processing.

use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;

use crate::{Req, Res};

/// Middleware processes requests before handlers.
#[async_trait]
pub trait Middleware<S = ()>: Send + Sync + 'static {
    /// Process request and optionally pass to next handler.
    async fn handle(&self, req: Req, state: Arc<S>, next: Next<S>) -> Res;
}

/// Continues to next middleware or handler.
pub struct Next<S = ()> {
    pub(crate) handler: Arc<dyn Fn(Req, Arc<S>) -> BoxFuture<Res> + Send + Sync>,
    pub(crate) state: Arc<S>,
}

type BoxFuture<T> = std::pin::Pin<Box<dyn Future<Output = T> + Send>>;

impl<S: 'static> Next<S> {
    /// Create next handler.
    #[inline]
    pub fn new(
        handler: Arc<dyn Fn(Req, Arc<S>) -> BoxFuture<Res> + Send + Sync>,
        state: Arc<S>,
    ) -> Self {
        Self { handler, state }
    }

    /// Pass request to next middleware or handler.
    #[inline]
    pub async fn run(self, req: Req) -> Res {
        (self.handler)(req, self.state).await
    }
}

/// Function-based middleware wrapper.
pub struct FnMiddleware<F>(pub F);

#[async_trait]
impl<F, Fut, S> Middleware<S> for FnMiddleware<F>
where
    F: Fn(Req, Arc<S>, Next<S>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    S: Send + Sync + 'static,
{
    async fn handle(&self, req: Req, state: Arc<S>, next: Next<S>) -> Res {
        (self.0)(req, state, next).await
    }
}
