use actix_web::{web, HttpResponse, Responder};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use mongodb::{bson::{doc, oid::ObjectId}, Client, Collection};
use chrono::NaiveDateTime;

use crate::jwt::generate_jwt;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: Vec<u8>,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: NaiveDateTime,
    pub refresh_token_expires_at: NaiveDateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub name: String,
}

pub async fn register_user(client: web::Data<mongodb::Client>, new_user: web::Json<User>) -> impl Responder {
    let db: mongodb::Database = client.database("sample_mfix");
    let collection: Collection<User> = db.collection("users");

    let new_user_data: User = new_user.into_inner();

    match collection.insert_one(new_user_data).await {
        Ok(insert_result) => HttpResponse::Ok().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn login_user(client: web::Data<Client>, login_info: web::Json<User>) -> impl Responder {
    let db: mongodb::Database = client.database("sample_mfix");
    let collection: Collection<User> = db.collection("users");

    let user: Option<User> = collection.find_one(doc! { "email": &login_info.email }).await.unwrap();

    if let Some(existing_user) = user {

        if existing_user.password == login_info.password {
            match generate_jwt(&existing_user.id.unwrap().to_string()) {
                Ok(token) => HttpResponse::Ok().json(token),
                Err(e) => HttpResponse::InternalServerError().body(format!("Error generating token: {}", e)),
            }
        } else {
            HttpResponse::Unauthorized().body("Invalid credentials")
        }
    } else {
        HttpResponse::Unauthorized().body("User not found")
    }
}

pub async fn get_users(client: web::Data<Client>) -> impl Responder {
    let db: mongodb::Database = client.database("sample_mfix");
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
    let db: mongodb::Database = client.database("sample_mfix");
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
    let db: mongodb::Database = client.database("sample_mfix");
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