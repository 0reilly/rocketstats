use actix_web::{web, App, HttpResponse, HttpServer, Responder, post};
use serde::Deserialize;
use std::fmt;
use deadpool_postgres::{Config, Pool};
use tokio_postgres::NoTls;
use actix_web::web::Data;
use log::{info, LevelFilter};
use env_logger::Builder;

impl fmt::Debug for EventData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventData")
            .field("url", &self.url)
            .field("referrer", &self.referrer)
            .field("user_agent", &self.device.user_agent)
            .finish()
    }
}

#[derive(Deserialize)]
struct Device {
    user_agent: String,
}

#[derive(Deserialize)]
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

async fn save_event_data(pool: &Pool, event_data: &EventData) -> Result<(), tokio_postgres::Error> {
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

#[post("/api/tracking/event")]
async fn handle_event(
    pool: web::Data<Pool>,
    event_data: web::Json<EventData>,
) -> impl Responder {
    println!("Received event data: {:?}", event_data);

    // Save the event data to the PostgreSQL database
    match save_event_data(&pool, &event_data).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Error saving event data: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    Builder::new()
        .filter(None, LevelFilter::Info)
        .init();

    let pool = create_pool().await;

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(handle_event)
    })
        .bind("0.0.0.0:8080")?
        .run();

    info!("Server started on http://127.0.0.1:8080");

    server.await
}
