use std::env;
use std::str::FromStr;
use mongodb::{bson::doc, options::ClientOptions, Client};
use serde::Deserialize;
use tide::{Request, Response, StatusCode};
use serde_json::Value;
use chrono::{DateTime, Utc};
use chrono_tz::US::Eastern;
use anyhow::{Context};
use tide::http::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use reqwest::Client as ReqwestClient;


#[derive(Debug, Deserialize)]
struct EventData {
    domain: String,
    url: String,
    referrer: String,
    device: Device,
}

#[derive(Debug, Deserialize)]
struct Device {
    user_agent: String,
}

#[tokio::main]
async fn main() -> tide::Result<()> {
    tide::log::start();

    let mongo_username = env::var("MONGO_USERNAME").expect("MONGO_USERNAME must be set");
    let mongo_password = env::var("MONGO_PASSWORD").expect("MONGO_PASSWORD must be set");
    let mongo_host = env::var("MONGO_HOST").expect("MONGO_HOST must be set");

    let mut client_options = ClientOptions::parse(format!("mongodb://{}:{}@{}", mongo_username, mongo_password, mongo_host)).await?;
    client_options.app_name = Some("rocketstats-backend".to_string());
    let client = Client::with_options(client_options)?;
    let db = client.database("rocketstats");

    let mut app = tide::new();

    let cors = CorsMiddleware::new()
        .allow_methods(HeaderValue::from_str("GET, POST, PUT, DELETE, OPTIONS").unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);


    app.at("/static").serve_dir("static/")?;
    app.at("/api/tracking/event").post(move |req: Request<()>| handle_event(req, db.clone()));


    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

async fn fetch_location_data(ip: &str) -> anyhow::Result<Value> {
    let client = ReqwestClient::new();
    let response = client
        .get(&format!("http://ip-api.com/json/{}", ip))
        .send()
        .await
        .context("Failed to fetch location data")?
        .text()
        .await
        .context("Failed to read response text")?;

    let location_data: Value = serde_json::from_str(&response)
        .context("Failed to parse location data")?;
    Ok(location_data)
}

async fn handle_event(mut req: Request<()>, db: mongodb::Database) -> tide::Result {
    let ip = req
        .remote()
        .and_then(|addr_str| addr_str.parse::<std::net::SocketAddr>().ok())
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| String::from("Unknown"));

    let event_data: EventData = req.body_json().await?;

    let location_data = fetch_location_data(&ip)
        .await
        .map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))?;


    let utc_now: DateTime<Utc> = Utc::now();
    let est_now = utc_now.with_timezone(&Eastern);
    // Get the city, region (state), and country
    let city = location_data["city"].as_str().unwrap_or("Unknown");
    let region = location_data["region"].as_str().unwrap_or("Unknown");
    let country = location_data["country"].as_str().unwrap_or("Unknown");

    println!(
        "{} - User Agent: {:?}",
        est_now.format("%Y-%m-%d %H:%M:%S").to_string(),
        event_data.device.user_agent
    );
    println!("   - Referrer: {:?}", event_data.referrer);
    println!("   - URL: {:?}", event_data.url);

    let events = db.collection("events");
    let document = doc! {
        "domain": event_data.domain,
        "url": event_data.url,
        "referrer": event_data.referrer,
        "user_agent": event_data.device.user_agent,
        "country": country,
        "region": region,
        "city": city,
        "timestamp": est_now.to_rfc3339(),
    };

    events.insert_one(document, None).await?;

    Ok(Response::new(StatusCode::Ok))
}
