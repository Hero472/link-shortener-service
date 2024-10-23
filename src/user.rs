use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use mongodb::{bson::{doc, oid::ObjectId, DateTime as BsonDateTime}, Client, Collection};
use chrono::{DateTime, Duration, Utc};
use std::time::SystemTime;
use serde_json::json;

use crate::jwt::{generate_jwt, generate_refresh_token};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token_expires_at: Option<BsonDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token_expires_at: Option<BsonDateTime>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub name: String,
}

pub async fn register_user(client: web::Data<mongodb::Client>, new_user: web::Json<User>) -> impl Responder {
    let db: mongodb::Database = client.database("shortener_link");
    let collection: Collection<User> = db.collection("users");

    let mut new_user_data: User = new_user.into_inner();

    match hash(&new_user_data.password, DEFAULT_COST) {
        Ok(hashed_password) => {
            new_user_data.password = hashed_password;
        },
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash the password")
    }

    match collection.insert_one(new_user_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn login_user(client: web::Data<Client>, login_info: web::Json<UserLogin>) -> impl Responder {
    let db: mongodb::Database = client.database("shortener_link");
    let collection: Collection<User> = db.collection("users");

    // Find user by email
    let user: Option<User> = collection
        .find_one(doc! { "email": &login_info.email })
        .await
        .unwrap();

    if let Some(existing_user) = user {
        // Verify the password
        match verify(&login_info.password, &existing_user.password) {
            Ok(is_valid) => {
                if is_valid {
                    // Generate access token
                    match generate_jwt(&existing_user.id.unwrap().to_string()) {
                        Ok(access_token) => {
                            // Generate refresh token
                            match generate_refresh_token(&existing_user.id.unwrap().to_string()) {
                                Ok(refresh_token) => {
                                    // Set expiration times
                                    let access_token_expires_at: DateTime<Utc> = Utc::now() + Duration::minutes(15); // 15 minutes
                                    let refresh_token_expires_at: DateTime<Utc> = Utc::now() + Duration::days(7); // 7 days

                                    // Convert DateTime<Utc> to SystemTime
                                    let access_token_expires_at_system_time: SystemTime = access_token_expires_at.into();
                                    let refresh_token_expires_at_system_time: SystemTime = refresh_token_expires_at.into();

                                    // Convert SystemTime to BSON DateTime
                                    let access_token_expires_at_bson: BsonDateTime = BsonDateTime::from(access_token_expires_at_system_time);
                                    let refresh_token_expires_at_bson: BsonDateTime = BsonDateTime::from(refresh_token_expires_at_system_time);

                                    // Update user document with tokens and expiration times
                                    let update_result = collection.update_one(
                                        doc! { "email": &existing_user.email },
                                        doc! {
                                            "$set": {
                                                "access_token": &access_token,
                                                "refresh_token": &refresh_token,
                                                "access_token_expires_at": &access_token_expires_at_bson,
                                                "refresh_token_expires_at": &refresh_token_expires_at_bson,
                                            }
                                        },
                                    ).await;

                                    match update_result {
                                        Ok(_) => HttpResponse::Ok().json(
                                            json!({
                                                "access_token": access_token,
                                                "refresh_token": refresh_token,
                                                "access_token_expires_at": access_token_expires_at,
                                                "refresh_token_expires_at": refresh_token_expires_at,
                                        })),
                                        Err(e) => HttpResponse::InternalServerError().body(format!("Error updating user: {}", e)),
                                    }
                                }
                                Err(e) => HttpResponse::InternalServerError().body(format!("Error generating refresh token: {}", e)),
                            }
                        }
                        Err(e) => HttpResponse::InternalServerError().body(format!("Error generating access token: {}", e)),
                    }
                } else {
                    HttpResponse::Unauthorized().body("Invalid credentials")
                }
            }
            Err(_) => HttpResponse::InternalServerError().body("Error verifying password"),
        }
    } else {
        HttpResponse::Unauthorized().body("User not found")
    }
}


pub async fn get_users(client: web::Data<Client>) -> impl Responder {
    let db: mongodb::Database = client.database("shortener_link");
    let collection: Collection<User> = db.collection("users");

    let mut cursor: mongodb::Cursor<User> = collection.find(doc! {}).await.unwrap();
    let mut users: Vec<User> = Vec::new();

    while let Some(result) = cursor.next().await {
        match result {
            Ok(user) => users.push(user),
            Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e))
        }
    }

    HttpResponse::Ok().json(users)
}

pub async fn update_user(
    client: web::Data<Client>,
    user_id: web::Path<String>,
    new_name: web::Json<UpdateUser>,
) -> impl Responder {
    let db: mongodb::Database = client.database("shortener_link");
    let collection: Collection<User> = db.collection("users");

    let object_id: ObjectId = match ObjectId::parse_str(&user_id.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID"),
    };

    let filter: mongodb::bson::Document = doc! { "_id": object_id };
    let update: mongodb::bson::Document = doc! { "$set": { "name": &new_name.name } };

    match collection.update_one(filter, update).await {
        Ok(update_result) => {
            if update_result.matched_count > 0 {
                HttpResponse::Ok().body("User updated successfully")
            } else {
                HttpResponse::NotFound().body("User not found")
            }
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn remove_user(client: web::Data<Client>, user_id: web::Path<String>) -> impl Responder {
    let db: mongodb::Database = client.database("shortener_link");
    let collection: Collection<User> = db.collection("users");

    let object_id: ObjectId = match ObjectId::parse_str(&user_id.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID")
    };

    let filter: mongodb::bson::Document = doc! {"_id": object_id };

    match collection.delete_one(filter).await {
        Ok(remove_result) => {
            if remove_result.deleted_count == 1 {
                HttpResponse::Ok().body("User removed successfully")
            } else {
                HttpResponse::NotFound().body("User not found")
            }
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e))
    }

}