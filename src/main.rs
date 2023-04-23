use serde::Deserialize;
use std::convert::Infallible;
use warp::{Filter, Reply};
use deadpool_postgres::{Config, Pool};
use tokio_postgres::NoTls;
use std::sync::Arc;
use warp::fs::dir;

#[derive(Deserialize, Debug)]
struct Device {
    user_agent: String,
}

#[derive(Deserialize, Debug)]
struct EventData {
    url: String,
    referrer: String,
    device: Device,
}

async fn create_pool() -> Pool {
    let mut cfg = Config::default();
    cfg.dbname = Some("mydb".to_string());
    cfg.user = Some("myuser".to_string());
    cfg.password = Some("mypassword".to_string());
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);

    let pool = cfg.create_pool(NoTls).unwrap();
    pool
}

async fn save_event_data(pool: &Pool, event_data: EventData) -> Result<(), tokio_postgres::Error> {
    let client = pool.get().await.unwrap();
    client
        .execute(
            "INSERT INTO event_tracking (url, referrer, user_agent) VALUES ($1, $2, $3)",
            &[
                &event_data.url,
                &event_data.referrer,
                &event_data.device.user_agent,
            ],
        )
        .await?;

    Ok(())
}

async fn handle_event(event_data: EventData, pool: Arc<Pool>) -> Result<impl Reply, Infallible> {
    println!("Received event data: {:?}", event_data);

    // Save the event data to the PostgreSQL database
    match save_event_data(&pool, event_data).await {
        Ok(_) => Ok(warp::reply::with_status("OK", warp::http::StatusCode::OK)),
        Err(e) => {
            println!("Error saving event data: {:?}", e);
            Ok(warp::reply::with_status(
                "Internal Server Error",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize the logger
    env_logger::init();

    let pool = Arc::new(create_pool().await);

    let event_route = warp::path!("api" / "tracking" / "event")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || Arc::clone(&pool)))
        .and_then(|event_data, pool| handle_event(event_data, pool));

    let static_route = warp::path("api").and(warp::path("tracking")).and(dir("./static"));

    let routes = event_route.or(static_route);

    println!("Server started on http://127.0.0.1:8080");

    //set the port to use on vps
    warp::serve(routes).run(([91, 107, 211, 239], 8080)).await;
}
