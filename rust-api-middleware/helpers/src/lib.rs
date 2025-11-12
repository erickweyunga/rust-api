//! Middleware composition helpers for Rust API framework.
//!
//! ## Available Helpers
//!
//! - `CombinedMiddleware` - Combine multiple middleware into one
//! - `ConditionalMiddleware` - Execute middleware based on predicate
//! - `MiddlewareChain` - Builder pattern for middleware composition

use async_trait::async_trait;
use rust_api::{Middleware, Next, Req, Res};
use std::sync::Arc;

/// Combine multiple middleware into a single middleware chain.
pub struct CombinedMiddleware<S = ()> {
    middleware: Vec<Arc<dyn Middleware<S>>>,
}

impl<S> CombinedMiddleware<S> {
    pub fn new(middleware: Vec<Arc<dyn Middleware<S>>>) -> Self {
        Self { middleware }
    }

    pub fn with(mut self, mw: Arc<dyn Middleware<S>>) -> Self {
        self.middleware.push(mw);
        self
    }
}

#[async_trait]
impl<S: Send + Sync + 'static> Middleware<S> for CombinedMiddleware<S> {
    async fn handle(&self, req: Req, state: Arc<S>, next: Next<S>) -> Res {
        let mut chain = next;

        for mw in self.middleware.iter().rev() {
            let mw_clone = Arc::clone(mw);
            let current_chain = chain;
            let state_clone = Arc::clone(&state);

            chain = Next::new(
                Arc::new(move |req, state| {
                    let mw = Arc::clone(&mw_clone);
                    let next = Next::new(current_chain.handler.clone(), Arc::clone(&state_clone));
                    Box::pin(async move { mw.handle(req, state, next).await })
                }),
                Arc::clone(&state),
            );
        }

        chain.run(req).await
    }
}

/// Conditional middleware - only execute if predicate returns true.
pub struct ConditionalMiddleware<S, M, F> {
    middleware: M,
    predicate: F,
    _marker: std::marker::PhantomData<S>,
}

impl<S, M, F> ConditionalMiddleware<S, M, F> {
    pub fn new(middleware: M, predicate: F) -> Self {
        Self {
            middleware,
            predicate,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<S, M, F> Middleware<S> for ConditionalMiddleware<S, M, F>
where
    S: Send + Sync + 'static,
    M: Middleware<S>,
    F: Fn(&Req, &Arc<S>) -> bool + Send + Sync + 'static,
{
    async fn handle(&self, req: Req, state: Arc<S>, next: Next<S>) -> Res {
        if (self.predicate)(&req, &state) {
            self.middleware.handle(req, state, next).await
        } else {
            next.run(req).await
        }
    }
}

/// Builder for composing middleware chains.
pub struct MiddlewareChain<S = ()> {
    middleware: Vec<Arc<dyn Middleware<S>>>,
}

impl<S: Send + Sync + 'static> MiddlewareChain<S> {
    pub fn new() -> Self {
        Self {
            middleware: Vec::new(),
        }
    }

    pub fn add<M: Middleware<S>>(mut self, mw: M) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    pub fn when<M, F>(mut self, predicate: F, mw: M) -> Self
    where
        M: Middleware<S>,
        F: Fn(&Req, &Arc<S>) -> bool + Send + Sync + 'static,
    {
        self.middleware
            .push(Arc::new(ConditionalMiddleware::new(mw, predicate)));
        self
    }

    pub fn build(self) -> CombinedMiddleware<S> {
        CombinedMiddleware::new(self.middleware)
    }
}

impl<S: Send + Sync + 'static> Default for MiddlewareChain<S> {
    fn default() -> Self {
        Self::new()
    }
}
