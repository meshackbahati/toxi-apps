//! Metrics middleware: records every request into the global registry.

use toxi::prelude::*;
use toxi_core::{Error, ToxiRequest, ToxiResponse};
use toxi_utils::metrics::GLOBAL_METRICS;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::Service;

/// Records path, duration, and outcome per request.
#[derive(Clone)]
pub struct Recorder<S> {
    inner: S,
}

impl<S> Recorder<S> {
    /// Wrap the inner service.
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<ToxiRequest> for Recorder<S>
where
    S: Service<ToxiRequest, Response = ToxiResponse, Error = Error> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: ToxiRequest) -> Self::Future {
        let path = req.uri().path().to_string();
        let start = Instant::now();
        let fut = self.inner.call(req);
        Box::pin(async move {
            let res = fut.await;
            GLOBAL_METRICS.record_request(&path, start.elapsed().as_millis() as u64, res.is_ok());
            res
        })
    }
}
