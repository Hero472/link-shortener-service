use mongodb::{options::ClientOptions, Client};
use routes::public_routes;
use tonic::client;
use std::env;
use dotenv::dotenv;

use actix_web::{web, App, HttpServer};

mod user;
mod routes;
mod jwt;

use proto::User;

mod proto {
    tonic::include_proto!("user");
}

#[tokio::main]
async fn main() -> Result<(), Box<mongodb::error::Error>> {
    dotenv().ok();


    let db_uri = env::var("MONGODB_URI").expect("Expected MONGODB_URI in environment variables");

    let client: Client = match ClientOptions::parse(&db_uri).await {
        Ok(client_options) => match Client::with_options(client_options) {
            Ok(client) => {
                println!("Connected to MongoDB!");
                client
            }
            Err(err) => {
                eprintln!("Failed to initialize MongoDB client: {}", err);
                return Err(Box::new(err));
            }
        },
        Err(err) => {
            eprintln!("Failed to parse MongoDB URI: {}", err);
            return Err(Box::new(err));
        }
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .configure(public_routes)
    })
    .bind("0.0.0.0:8080")
    .expect("Failed to bind to 0.0.0.0:8080 - check if the port is already in use or if permissions are insufficient")
    .run()
    .await
    .expect("HTTP server encountered an error while running");
    
    Ok(())
}
