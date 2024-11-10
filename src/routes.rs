use actix_web::web;

use crate::user::{ get_users, login_user, refresh_user, register_user, remove_user, update_user};

pub fn public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/register")
            .route(web::post().to(register_user))
    );

    cfg.service(
        web::resource("/login")
            .route(web::post().to(login_user))
    );

    cfg.service(
        web::resource("/users")
            .route(web::get().to(get_users))
    );

    cfg.service(
        web::resource("/users/{id}")
            .route(web::patch().to(update_user))
            .route(web::delete().to(remove_user))
    );

    cfg.service(
        web::resource("/user/refresh/{id}")
            .route(web::patch().to(refresh_user))
    );
}