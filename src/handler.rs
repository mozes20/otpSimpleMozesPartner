use crate::{ticket, AppState};
use axum::{
    extract::State,
    http::{status, Response},
    Json,
};
use std::sync::Arc;

pub async fn create_ticket(
    State(app_state): State<Arc<AppState>>,
    payload: Option<Json<ticket::Ticket>>,
) -> Response<String> {
    if let Some(payload) = payload {
        // We got a valid JSON payload
        let ticket = payload.0;
        let ticket_functions = ticket::TicketFunctions::new(app_state.mongo.clone());
        let _ = ticket_functions.create_ticket(ticket).await;
        return Response::builder()
            .status(status::StatusCode::CREATED)
            .body("Ticket created!".into())
            .unwrap();
    } else {
        return Response::builder()
            .status(status::StatusCode::BAD_REQUEST)
            .body("Invalid JSON payload".into())
            .unwrap();
        // Payload wasn't valid JSON
    }
}
