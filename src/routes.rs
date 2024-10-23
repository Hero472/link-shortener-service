use actix_web::web;

use crate::{jwt::JwtMiddleware, user::{ get_users, login_user, register_user, remove_user, update_user}};

// Public routes for registration and login
pub fn public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/register")
            .route(web::post().to(register_user))
    );

    cfg.service(
        web::resource("/login")
            .route(web::post().to(login_user))
    );
}

// Protected routes for user management
pub fn protected_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
            .route(web::get().to(get_users))
            .wrap(JwtMiddleware) // Protect this route with JWT middleware
    );

    cfg.service(
        web::resource("/users/{id}")
            .route(web::put().to(update_user))
            .route(web::delete().to(remove_user))
            .wrap(JwtMiddleware) // Protect this route with JWT middleware
    );
}