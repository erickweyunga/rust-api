//! Router for organizing routes into groups
//!
//! Use [`Router`] to create nested route groups with their own middleware.

use hyper::Method;
use std::sync::Arc;

use crate::{Handler, Middleware, handler::IntoHandler, middleware::FnMiddleware};

type BoxedHandler<S> = Arc<dyn Handler<S>>;
type BoxedMiddleware<S> = Arc<dyn Middleware<S>>;

/// Route group for organizing endpoints
pub struct Router<S = ()> {
    routes: Vec<(Method, String, BoxedHandler<S>)>,
    middlewares: Vec<BoxedMiddleware<S>>,
    nested: Vec<(String, Router<S>)>,
}

impl<S: Send + Sync + 'static> Router<S> {
    /// Create new router
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
            nested: Vec::new(),
        }
    }

    /// Add GET route
    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::GET, path.to_string(), handler.into_handler()));
        self
    }

    /// Add POST route
    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::POST, path.to_string(), handler.into_handler()));
        self
    }

    /// Add PUT route
    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::PUT, path.to_string(), handler.into_handler()));
        self
    }

    /// Add DELETE route
    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::DELETE, path.to_string(), handler.into_handler()));
        self
    }

    /// Add PATCH route
    pub fn patch<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::PATCH, path.to_string(), handler.into_handler()));
        self
    }

    /// Add middleware layer
    pub fn layer<F, Fut>(mut self, middleware: F) -> Self
    where
        F: Fn(crate::Req, Arc<S>, crate::Next<S>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = crate::Res> + Send + 'static,
    {
        self.middlewares.push(Arc::new(FnMiddleware(middleware)));
        self
    }

    /// Nest another router at a path prefix
    pub fn nest(mut self, prefix: &str, router: Router<S>) -> Self {
        self.nested.push((prefix.to_string(), router));
        self
    }

    /// Flatten router into routes with full paths
    pub(crate) fn flatten(
        &self,
        prefix: &str,
    ) -> Vec<(Method, String, BoxedHandler<S>, Vec<BoxedMiddleware<S>>)> {
        let mut flattened = Vec::new();

        // Add direct routes
        for (method, path, handler) in &self.routes {
            let full_path = format!("{}{}", prefix, path);
            flattened.push((
                method.clone(),
                full_path,
                Arc::clone(handler),
                self.middlewares.clone(),
            ));
        }

        // Recursively flatten nested routers
        for (nested_prefix, nested_router) in &self.nested {
            let full_prefix = format!("{}{}", prefix, nested_prefix);
            let mut nested_routes = nested_router.flatten(&full_prefix);

            // Add parent middleware to nested routes
            for (_, _, _, middlewares) in &mut nested_routes {
                let mut combined = self.middlewares.clone();
                combined.append(middlewares);
                *middlewares = combined;
            }

            flattened.extend(nested_routes);
        }

        flattened
    }
}

impl<S> Default for Router<S>
where
    S: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
