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

    app.at("/api/tracking/event")
        .options(handle_options)
        .post(handle_event);

    app.at("/static/:file")
        .get(|req: Request<PgPool>| async move {
            let file = req.param("file")?;
            let path = format!("./static/{}", file);
            let body = async_std::fs::read(path).await?;
            Ok(Response::builder(StatusCode::Ok).body(tide::Body::from(body)).build())
        });


    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

async fn handle_options(_req: Request<PgPool>) -> tide::Result {
    let mut response = Response::new(StatusCode::Ok);
    response.insert_header("Access-Control-Allow-Origin", "*");
    response.insert_header("Access-Control-Allow-Methods", "POST, OPTIONS");
    response.insert_header("Access-Control-Allow-Headers", "Content-Type");
    Ok(response)
}


async fn handle_event(mut req: Request<PgPool>) -> tide::Result {
    let event_data: EventData = req.body_json().await?;

    //console log the event data (body is sent as stringified json)
    println!("{:?}", event_data);

    Ok(Response::new(StatusCode::Ok))
}
