//! Long-poll task events.
//!
//! POST /tasks publishes to the `tasks` bus. GET /events/next waits up
//! to 20 seconds for the next event and returns it as JSON.

use toxi::realtime::Event;
use toxi::json_response;
use toxi_core::{Error, FromRequest, Request, Response, Result, State};
use std::sync::Arc;

use crate::AppState;

/// Wait for the next task event.
pub async fn next(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let mut sub = state.bus.subscribe("tasks").await;
    match tokio::time::timeout(std::time::Duration::from_secs(20), sub.recv()).await {
        Ok(Ok(event)) => Ok(json_response!(event_to_json(&event))),
        Ok(Err(e)) => Err(Error::InternalServerError(e.to_string())),
        Err(_) => Ok(json_response!({ "events": [] })),
    }
}

fn event_to_json(event: &Event) -> impl serde::Serialize + '_ {
    #[derive(serde::Serialize)]
    struct TaskEvent<'a> {
        channel: &'a str,
        kind: String,
        data: &'a serde_json::Value,
    }
    TaskEvent {
        channel: &event.channel,
        kind: format!("{:?}", event.event_type),
        data: &event.data,
    }
}
