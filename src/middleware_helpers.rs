//! Middleware composition helpers
//!
//! Utilities for combining and composing middleware.

use crate::{Middleware, Next, Req, Res};
use async_trait::async_trait;
use std::sync::Arc;

/// Combine multiple middleware into a single middleware chain
///
/// # Example
///
/// ```ignore
/// let combined = CombinedMiddleware::new(vec![
///     Arc::new(LoggerMiddleware),
///     Arc::new(AuthMiddleware),
///     Arc::new(CorsMiddleware),
/// ]);
///
/// app.layer(combined);
/// ```
pub struct CombinedMiddleware<S = ()> {
    middleware: Vec<Arc<dyn Middleware<S>>>,
}

impl<S> CombinedMiddleware<S> {
    /// Create a new combined middleware
    pub fn new(middleware: Vec<Arc<dyn Middleware<S>>>) -> Self {
        Self { middleware }
    }

    /// Add another middleware to the chain
    pub fn with(mut self, mw: Arc<dyn Middleware<S>>) -> Self {
        self.middleware.push(mw);
        self
    }
}

#[async_trait]
impl<S: Send + Sync + 'static> Middleware<S> for CombinedMiddleware<S> {
    async fn handle(&self, req: Req, state: Arc<S>, next: Next<S>) -> Res {
        // Build nested middleware chain
        let mut chain = next;

        // Wrap each middleware around the chain (in reverse)
        for mw in self.middleware.iter().rev() {
            let mw_clone = Arc::clone(mw);
            let current_chain = chain;
            let state_clone = Arc::clone(&state);

            // Create a new Next that calls this middleware
            chain = Next::new(
                Arc::new(move |req, state| {
                    let mw = Arc::clone(&mw_clone);
                    let next = Next::new(current_chain.handler.clone(), Arc::clone(&state_clone));
                    Box::pin(async move { mw.handle(req, state, next).await })
                }),
                Arc::clone(&state),
            );
        }

        // Execute the chain
        chain.run(req).await
    }
}

/// Conditional middleware - only execute if predicate returns true
///
/// # Example
///
/// ```ignore
/// let conditional = ConditionalMiddleware::new(
///     LoggerMiddleware,
///     |req, _state| req.path().starts_with("/api")
/// );
///
/// app.layer(conditional);  // Only logs /api/* routes
/// ```
pub struct ConditionalMiddleware<S, M, F> {
    middleware: M,
    predicate: F,
    _marker: std::marker::PhantomData<S>,
}

impl<S, M, F> ConditionalMiddleware<S, M, F> {
    /// Create a new conditional middleware
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
        // Check condition
        if (self.predicate)(&req, &state) {
            // Execute middleware
            self.middleware.handle(req, state, next).await
        } else {
            // Skip middleware
            next.run(req).await
        }
    }
}

/// Builder for composing middleware chains
///
/// # Example
///
/// ```ignore
/// let chain = MiddlewareChain::new()
///     .add(LoggerMiddleware)
///     .add(CorsMiddleware)
///     .when(|req, _| req.path().starts_with("/api"))
///     .add(AuthMiddleware)
///     .build();
///
/// app.layer(chain);
/// ```
pub struct MiddlewareChain<S = ()> {
    middleware: Vec<Arc<dyn Middleware<S>>>,
}

impl<S: Send + Sync + 'static> MiddlewareChain<S> {
    /// Create a new middleware chain builder
    pub fn new() -> Self {
        Self {
            middleware: Vec::new(),
        }
    }

    /// Add middleware to the chain
    pub fn add<M: Middleware<S>>(mut self, mw: M) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    /// Add conditional middleware
    pub fn when<M, F>(mut self, predicate: F, mw: M) -> Self
    where
        M: Middleware<S>,
        F: Fn(&Req, &Arc<S>) -> bool + Send + Sync + 'static,
    {
        self.middleware
            .push(Arc::new(ConditionalMiddleware::new(mw, predicate)));
        self
    }

    /// Build the final combined middleware
    pub fn build(self) -> CombinedMiddleware<S> {
        CombinedMiddleware::new(self.middleware)
    }
}

impl<S: Send + Sync + 'static> Default for MiddlewareChain<S> {
    fn default() -> Self {
        Self::new()
    }
}
