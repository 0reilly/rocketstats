use std::env;
use std::str::FromStr;
use mongodb::{bson::doc, options::ClientOptions, Client, options::FindOptions, bson};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use chrono_tz::US::Eastern;
use anyhow::{Context};
use tide::http::headers::HeaderValue;
use surf::Client as surfClient;
use tide::log::LevelFilter;
use tide::security::{CorsMiddleware, Origin};
use std::collections::HashMap;
use futures_util::stream::StreamExt;

use tide::{Request, Response, StatusCode};

#[derive(Debug, Deserialize, Serialize)]
struct EventData {
    domain: String,
    url: String,
    referrer: String,
    device: Device,
}

#[derive(Debug, Deserialize, Serialize)]
struct Device {
    user_agent: String,
}

#[derive(Serialize, Deserialize)] // Add Deserialize to VisitorStats
struct VisitorStats {
    visitor_count: usize,
    pageviews: HashMap<String, usize>,
    locations: HashMap<String, usize>,
    sources: HashMap<String, usize>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = env::var("PORT")
        .ok()
        .and_then(|port| u16::from_str(&port).ok())
        .unwrap_or(8080);
    tide::log::with_level(LevelFilter::Info);

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
    app.at("/api/tracking/event").post({
        let db = db.clone();
        move |req: Request<()>| handle_event(req, db.clone())
    });

    app.at("/:domain").get({
        let db = db.clone();
        move |req: Request<()>| {
            let db = db.clone();
            async move {
                let domain = req.param("domain")?.to_string();
                fetch_all_statistics(db, domain).await
            }
        }
    });

    app.at("/api/all-data").get({
        let db = db.clone();
        move |req: Request<()>| get_all_data(req, db.clone())
    });


    app.listen(format!("0.0.0.0:{}", port)).await?;
    Result::<(), anyhow::Error>::Ok(())
}

async fn get_all_data(_req: Request<()>, db: mongodb::Database) -> tide::Result {
    let events = db.collection("events");

    let mut cursor = events.find(None, None).await?;

    //print cursor to string
    println!("{:?}", cursor);

    let mut all_data = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                if let Ok(event_data) = bson::from_bson::<EventData>(bson::Bson::Document(document)) {
                    all_data.push(event_data);
                }
            }
            Err(_) => {
                return Ok(Response::new(StatusCode::InternalServerError));
            }
        }
    }

    let json_string = serde_json::to_string(&all_data)?;
    let body = tide::Body::from_json(&json_string)?;
    let mut response = Response::new(StatusCode::Ok);
    response.set_body(body);
    Ok(response)
}


async fn fetch_all_statistics(db: mongodb::Database, domain: String) -> tide::Result {
    let events = db.collection("events");

    let filter = doc! {
        "domain": domain
    };

    let options = FindOptions::builder().sort(doc! { "timestamp": -1 }).build();
    let mut cursor = events.find(filter, options).await?;

    let mut visitor_stats = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                if let Ok(stat) = bson::from_bson::<VisitorStats>(bson::Bson::Document(document)) {
                    visitor_stats.push(stat);
                }
            }
            Err(_) => {
                return Ok(Response::new(StatusCode::InternalServerError));
            }
        }
    }

    let json_string = serde_json::to_string(&visitor_stats)?;
    let body = tide::Body::from_json(&json_string)?;
    let mut response = Response::new(StatusCode::Ok);
    response.set_body(body);
    Ok(response)
}

async fn fetch_location_data(ip: &str) -> anyhow::Result<Value> {
    let surf_client = surfClient::new();
    let response = surf_client
        .get(&format!("http://ip-api.com/json/{}", ip.to_string()))
        .recv_string()
        .await
        .map_err(anyhow::Error::msg)?;

    let location_data: Value = serde_json::from_str(&response)
        .context("Failed to parse location data")?;
    Ok(location_data)
}

async fn handle_event(mut req: Request<()>, db: mongodb::Database) -> tide::Result {
    let ip = req.header("X-Forwarded-For")
        .and_then(|values| values.get(0))
        .map(|value| value.as_str().to_owned())
        .unwrap_or_else(|| String::from("Unknown"));

    println!("   - IP: {:?}", ip);

    let event_data: EventData = req.body_json().await?;

    let location_data = fetch_location_data(&ip)
        .await
        .map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))?;

    let utc_now: DateTime<Utc> = Utc::now();
    let est_now = utc_now.with_timezone(&Eastern);
    let city = location_data["city"].as_str().unwrap_or("Unknown");
    let region = location_data["region"].as_str().unwrap_or("Unknown");
    let country = location_data["country"].as_str().unwrap_or("Unknown");
    println!(
        "{} - {} - {} - {}",
        est_now.format("%Y-%m-%d %H:%M:%S").to_string(),
        country,
        region,
        city
    );

    println!(
        "{} - User Agent: {:?}",
        est_now.format("%Y-%m-%d %H:%M:%S").to_string(),
        event_data.device.user_agent.to_string()
    );
    println!("   - Referrer: {:?}", event_data.referrer.to_string());
    println!("   - URL: {:?}", event_data.url.to_string());

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

    println!("Inserting document: {:?}", document);


    let insert_result = events.insert_one(document, None).await?;
    println!("Insert result: {:?}", insert_result);

    Ok(Response::new(StatusCode::Ok))
}
