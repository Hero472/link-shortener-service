use actix_web::{http::StatusCode, test, web, App};
use api::*;
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use user::{login_user, register_user, User, UserLogin, Role};

mod common;

async fn setup() -> Client {
    let client_options: ClientOptions = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client: Client = Client::with_options(client_options).unwrap();

    let db: mongodb::Database = client.database("test_shortener_link");
    
    db.create_collection("users").await.unwrap();

    client
}

async fn teardown(client: &Client) {

    let db: mongodb::Database = client.database("test_shortener_link");
    db.drop().await.unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: String,
    pub refresh_token_expires_at: String,
}

#[actix_rt::test]
async fn test_register_user() {

    let client: Client = setup().await;
    
    let new_user: User = User {
        id: None,
        username: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        role: user::Role::User,
        access_token: None,
        refresh_token: None,
        access_token_expires_at: None,
        refresh_token_expires_at: None,
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(web::resource("/register").route(web::post().to(register_user)))
    ).await;

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&new_user)
        .to_request();

    let resp: actix_web::dev::ServiceResponse= test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CREATED);

    teardown(&client);
}

#[actix_rt::test]
async fn test_login_user() {

    let client: Client = setup().await;
    
    let new_user: User = User {
        id: None,
        username: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        role: user::Role::User,
        access_token: None,
        refresh_token: None,
        access_token_expires_at: None,
        refresh_token_expires_at: None,
    };


    let user: UserLogin = UserLogin {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(web::resource("/register").route(web::post().to(register_user)))
            .service(web::resource("/login").route(web::post().to(login_user)))
    ).await;

    let req1 = test::TestRequest::post()
        .uri("/register")
        .set_json(&new_user)
        .to_request();

    let req2 = test::TestRequest::post()
        .uri("/login")
        .set_json(&user)
        .to_request();

    let _: actix_web::dev::ServiceResponse= test::call_service(&app, req1).await;
    let resp2: actix_web::dev::ServiceResponse= test::call_service(&app, req2).await;

    assert_eq!(resp2.status(), StatusCode::OK);

    let body: web::Bytes = test::read_body(resp2).await;
    println!("{:?}", body);
    teardown(&client);
}

#[actix_rt::test]
async fn test_get_users() {

    let client: Client = setup().await;
    
    let new_user: User = User {
        id: None,
        username: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        role: user::Role::User,
        access_token: None,
        refresh_token: None,
        access_token_expires_at: None,
        refresh_token_expires_at: None,
    };


    let user: UserLogin = UserLogin {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(web::resource("/register").route(web::post().to(register_user)))
            .service(web::resource("/login").route(web::post().to(login_user)))
    ).await;

    let req1 = test::TestRequest::post()
        .uri("/register")
        .set_json(&new_user)
        .to_request();

    let req2 = test::TestRequest::post()
        .uri("/login")
        .set_json(&user)
        .to_request();

    let req3 = test::TestRequest::post()
    .uri("/users")
    .set_json(&user)
    .to_request();
    
    let _: actix_web::dev::ServiceResponse= test::call_service(&app, req1).await;
    let resp2: actix_web::dev::ServiceResponse= test::call_service(&app, req2).await;

    println!("{:?}", resp2);
    let body: web::Bytes = test::read_body(resp2).await;

}