use crate::util::password::verify_password;
use actix_web::{HttpResponse, Responder, get, post, web};
use common::time::get_duration_since_unix;
use database::account::AccountDoc;
use database::srtools::{SRToolsData, SRToolsDoc, SRToolsMetaDoc};
use database::{DB_NAME, MongoClient};
use serde::Deserialize;

#[get("/srtools/user/{username}")]
pub async fn get_json(
    username: web::Path<String>,
    mongo_client: web::Data<MongoClient>,
) -> impl Responder {
    let db = mongo_client.database(DB_NAME);
    let st_coll = SRToolsDoc::get_collection(&db);

    match SRToolsDoc::fetch_by_username(&st_coll, &username).await {
        Ok(Some(v)) => HttpResponse::Ok().json(v),
        Ok(None) => HttpResponse::Ok().json("\"not found\""),
        Err(e) => {
            tracing::error!("Fetching SRToolsDoc by username: {}", e);
            HttpResponse::Ok().json("\"internal error\"")
        }
    }
}

#[derive(Deserialize)]
struct SRToolSyncRequest {
    username: String,
    password: String,
    data: Option<SRToolsData>,
}

#[post("/srtools/sync")]
pub async fn post_sync(
    request: web::Json<SRToolSyncRequest>,
    mongo_client: web::Data<MongoClient>,
) -> impl Responder {
    let db = mongo_client.database(DB_NAME);
    let ac_coll = AccountDoc::get_collection(&db);
    let account = match AccountDoc::fetch_by_username(&ac_coll, &request.username).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return HttpResponse::NotFound()
                .body(r#"{"status":404,"message":"account not found"}"#);
        }
        Err(e) => {
            tracing::error!("Fetching account by username: {}", e);
            return HttpResponse::InternalServerError()
                .body(r#"{"status":500,"message":"internal error"}"#);
        }
    };

    match verify_password(&request.password, &account.password_hash) {
        Ok(false) => {
            return HttpResponse::Unauthorized()
                .body(r#"{"status":401,"message":"wrong password"}"#);
        }
        Err(e) => {
            tracing::error!("Verifying password: {}", e);
            return HttpResponse::InternalServerError()
                .body(r#"{"status":500,"message":"internal error"}"#);
        }
        _ => {}
    };

    let stm_coll = SRToolsMetaDoc::get_collection(&db);
    let meta = match SRToolsMetaDoc::fetch_by_username(&stm_coll, &request.username).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return HttpResponse::NotFound()
                .body(r#"{"status":404,"message":"SRToolsMetaDoc not found"}"#);
        }
        Err(e) => {
            tracing::error!("Fetching SRToolsMetaDoc by username: {}", e);
            return HttpResponse::InternalServerError()
                .body(r#"{"status":500,"message":"internal error"}"#);
        }
    };

    let now_minutes = (get_duration_since_unix().as_secs() / 60) as u32;
    if meta.next_sync_allowed > now_minutes {
        return HttpResponse::TooManyRequests().body(format!(
            r#"{{"status":429,"message":"sync is on cooldown for {} more minutes"}}"#,
            meta.next_sync_allowed - now_minutes
        ));
    }

    if let Some(v) = &request.data {
        let st_coll = SRToolsDoc::get_collection(&db);
        if let Err(e) = SRToolsDoc::set_srtools_by_username(&st_coll, &request.username, v).await {
            tracing::error!("Setting srtoolsdata by username: {}", e);
        }

        if let Err(e) =
            SRToolsMetaDoc::update_next_sync_for_username(&stm_coll, &request.username).await
        {
            tracing::error!("Updating next sync: {}", e);
        }
    }

    HttpResponse::Ok().body(r#"{"status":200,"message":"success"}"#)
}
