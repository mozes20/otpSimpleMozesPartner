use dotenv::dotenv;
use handler::create_ticket;
use mongodb::{error::Result, Client};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Mutex};
use tokio::fs::File;
mod handler;
mod ticket;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tokio::io::AsyncReadExt;

pub struct AppState {
    mongo: Client,
    reservations: Arc<Mutex<HashMap<(u64, u64), bool>>>,
}

#[derive(Deserialize)]
struct ReserveRequest {
    event_id: u64,
    seat_id: u64,
}

#[derive(Serialize)]
struct SuccessResponse {
    success: bool,
    reservation_id: u64,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error_code: i32,
}

async fn get_events(State(_app_state): State<Arc<AppState>>) -> Response<String> {
    let mut file = File::open("getEvents.json").await.unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.unwrap();
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .body(contents)
        .unwrap()
}

async fn get_event(
    State(_app_state): State<Arc<AppState>>,
    Path(event_id): Path<u64>,
) -> Response<String> {
    let file_name = match event_id {
        1 => "getEvent1.json",
        2 => "getEvent2.json",
        3 => "getEvent3.json",
        _ => {
            return Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .body("Event not found".into())
                .unwrap()
        }
    };

    let mut file = File::open(file_name).await.unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.unwrap();
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .body(contents)
        .unwrap()
}

async fn reserve(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<ReserveRequest>,
) -> Response {
    let mut reservations = app_state.reservations.lock().unwrap();

    if let Some(&is_reserved) = reservations.get(&(payload.event_id, payload.seat_id)) {
        if is_reserved {
            return Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .body("Seat already taken".into())
                .unwrap();
        }
    }

    // Mark the seat as reserved
    reservations.insert((payload.event_id, payload.seat_id), true);

    // Generate a reservation ID
    let reservation_id = rand::thread_rng().gen::<u64>();

    return Response::builder()
        .status(axum::http::StatusCode::CREATED)
        .body(
            serde_json::to_string(&SuccessResponse {
                success: true,
                reservation_id,
            })
            .unwrap()
            .into(),
        )
        .unwrap();
}

async fn connect_to_mongo() -> Result<Client> {
    // Load environment variables from .env file
    dotenv().ok();

    // Load the MongoDB connection string from an environment variable:
    let client_uri =
        env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");

    // A Client is needed to connect to MongoDB:
    let client = Client::with_uri_str(client_uri).await?;
    //let mut ticket = ticket::TicketFunctions::new(client.clone());

    Ok(client)
}

#[tokio::main]
async fn main() {
    // Connect to MongoDB
    let client = connect_to_mongo()
        .await
        .expect("Failed to connect to MongoDB");

    let app = Router::new()
        .route("/ticket", post(create_ticket))
        .route("/getEvents", get(get_events))
        .route("/getEvent/:event_id", get(get_event))
        .route("/reserve", post(reserve))
        .with_state(Arc::new(AppState {
            mongo: client.clone(),
            reservations: Arc::new(Mutex::new(HashMap::new())),
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3500").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
