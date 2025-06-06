use crate::util::password::hash_password;
use crate::util::token::generate_token;
use actix_web::{HttpResponse, Responder, get, http::header::ContentType, post, web};
use database::account::{AccountDoc, AccountMetaDoc};
use database::srtools::{SRToolsDoc, SRToolsMetaDoc};
use database::{DB_NAME, MongoClient};
use serde::Deserialize;

const REGISTER_PAGE: &str = include_str!("../../include/register.html");

#[get("/account/register")]
pub async fn get_register() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(REGISTER_PAGE)
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[post("/account/register")]
pub async fn post_register(
    request: web::Json<RegisterRequest>,
    mongo_client: web::Data<MongoClient>,
) -> impl Responder {
    if request.username.len() < 4 || request.username.len() > 12 {
        return HttpResponse::BadRequest().body("Username length must be between 4-12");
    }
    if request.password.len() < 4 {
        return HttpResponse::BadRequest().body("Password length must be over 4");
    }

    let db = mongo_client.database(DB_NAME);
    let ac_coll = AccountDoc::get_collection(&db);

    match AccountDoc::check_username_taken(&ac_coll, &request.username).await {
        Ok(true) => {
            return HttpResponse::InternalServerError().body("Username is already taken");
        }
        Err(e) => {
            tracing::error!("Checking username taken: {}", e);
            return HttpResponse::InternalServerError().body("Internal server error.");
        }
        _ => {}
    }

    let password_hash = match hash_password(&request.password) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Hashing password: {}", e);
            return HttpResponse::InternalServerError().body("Internal server error.");
        }
    };

    let token = generate_token();
    let uid = match AccountMetaDoc::get_next_uid(&AccountMetaDoc::get_collection(&db)).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Getting next uid: {}", e);
            return HttpResponse::InternalServerError().body("Internal server error.");
        }
    };

    let new_account = AccountDoc {
        uid,
        username: request.username.clone(),
        password_hash,
        token,
        is_banned: false,
        ban_reason: None,
    };

    if let Err(e) = new_account.insert_to_collection(&ac_coll).await {
        tracing::error!("Inserting new account: {}", e);
        return HttpResponse::InternalServerError().body("Internal server error.");
    }

    match (SRToolsDoc {
        uid,
        username: request.username.clone(),
        data: None,
    })
    .insert_to_collection(&SRToolsDoc::get_collection(&db))
    .await
    {
        Err(e) => {
            tracing::error!("Inserting srtoolsdoc: {}", e);
            return HttpResponse::InternalServerError().body("Internal server error.");
        }
        _ => {}
    };
    match (SRToolsMetaDoc {
        username: request.username.clone(),
        next_sync_allowed: 0,
        next_export_allowed: 0,
    })
    .insert_to_collection(&SRToolsMetaDoc::get_collection(&db))
    .await
    {
        Err(e) => {
            tracing::error!("Inserting srtoolsmetadoc: {}", e);
            return HttpResponse::InternalServerError().body("Internal server error.");
        }
        _ => {}
    }

    HttpResponse::Ok().body(format!("Register success! Your uid is {}", uid))
}
