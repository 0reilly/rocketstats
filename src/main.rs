use sqlx::PgPool;
use std::env;
use tide::{Request, Response, StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EventData {
    url: String,
    referrer: String,
    device: Device,
}

#[derive(Debug, Deserialize)]
struct Device {
    user_agent: String,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await?;

    let mut app = tide::with_state(pool);

    app.at("/events").post(handle_event);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn handle_event(mut req: Request<PgPool>) -> tide::Result {
    let event_data: EventData = req.body_json().await?;

    sqlx::query!(
        "INSERT INTO events (url, referrer, user_agent) VALUES ($1, $2, $3)",
        event_data.url,
        event_data.referrer,
        event_data.device.user_agent
    )
        .execute(&req.state().clone())
        .await?;

    Ok(Response::new(StatusCode::Ok))
}
