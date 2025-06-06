use crate::util::token::{generate_token, should_refresh_token};
use actix_web::{Responder, post, web};
use database::account::AccountDoc;
use database::{DB_NAME, MongoClient};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginPasswordRequest {
    // this is username
    account: String,
    // these needs a patch to work with, so we wont work with it
    // password: String,
    // is_crypto: bool,
}

#[post("/{product}/mdk/shield/api/login")]
pub async fn post_login_by_password(
    request: web::Json<LoginPasswordRequest>,
    mongo_client: web::Data<MongoClient>,
) -> impl Responder {
    let db = mongo_client.database(DB_NAME);
    let ac_coll = AccountDoc::get_collection(&db);
    let account = match AccountDoc::fetch_by_username(&ac_coll, &request.account).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return r#"{"data":{},"message":"account doesn't exist","retcode":1005}"#.to_string();
        }
        Err(e) => {
            tracing::error!("Fetching account by uid: {}", e);
            return r#"{"data":{},"message":"internal error","retcode":2}"#.to_string();
        }
    };

    let new_token: String;
    match should_refresh_token(&account.token) {
        Ok(true) => {
            new_token = generate_token();
            if let Err(e) = AccountDoc::update_token_by_uid(&ac_coll, account.uid, &new_token).await
            {
                tracing::error!("Updating token: {}", e);
                return r#"{"data":{},"message":"internal error","retcode":2}"#.to_string();
            }
        }
        Err(e) => {
            tracing::error!("Refreshing token: {}", e);
            return r#"{"data":{},"message":"internal error","retcode":2}"#.to_string();
        }
        _ => new_token = account.token,
    }

    format!(
        r#"{{"data":{{"account":{{"area_code":"**","country":"ID","email":"{}@railgun.ps","is_email_verify":"1","token":"{}","uid":"{}"}},"device_grant_required":false,"reactivate_required":false,"realperson_required":false,"safe_mobile_required":false}},"message":"OK","retcode":0}}"#,
        account.username, new_token, account.uid
    )
}

#[derive(Deserialize)]
pub struct ShieldVerifyRequest {
    uid: String,
    token: String,
}

#[post("/{product}/mdk/shield/api/verify")]
pub async fn post_login_by_token(
    request: web::Json<ShieldVerifyRequest>,
    mongo_client: web::Data<MongoClient>,
) -> impl Responder {
    let Ok(uid) = request.uid.parse::<u32>() else {
        return r#"{"data":{},"message":"bad request","retcode":5}"#.to_string();
    };
    let db = mongo_client.database(DB_NAME);
    let ac_coll = AccountDoc::get_collection(&db);
    let account = match AccountDoc::fetch_by_uid(&ac_coll, uid).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return r#"{"data":{},"message":"account doesn't exist","retcode":1005}"#.to_string();
        }
        Err(e) => {
            tracing::error!("Fetching account by uid: {}", e);
            return r#"{"data":{},"message":"internal error","retcode":2}"#.to_string();
        }
    };

    if &account.token != &request.token {
        return r#"{"data":{},"message":"token mismatch","retcode":1005}"#.to_string();
    }

    let new_token: String;
    match should_refresh_token(&request.token) {
        Ok(true) => {
            new_token = generate_token();
            if let Err(e) = AccountDoc::update_token_by_uid(&ac_coll, account.uid, &new_token).await
            {
                tracing::error!("Updating token: {}", e);
                return r#"{"data":{},"message":"internal error","retcode":2}"#.to_string();
            }
        }
        Err(e) => {
            tracing::error!("Refreshing token: {}", e);
            return r#"{"data":{},"message":"internal error","retcode":2}"#.to_string();
        }
        _ => new_token = request.token.clone(),
    }

    format!(
        r#"{{"data":{{"account":{{"area_code":"**","country":"ID","email":"{}@railgun.ps","is_email_verify":"1","token":"{}","uid":"{}"}},"device_grant_required":false,"reactivate_required":false,"realperson_required":false,"safe_mobile_required":false}},"message":"OK","retcode":0}}"#,
        account.username, new_token, account.uid
    )
}

#[derive(Deserialize)]
struct GrantLoginRequest {
    data: String,
}

#[derive(Deserialize)]
struct GrantLoginData {
    token: String,
    uid: String,
}

impl GrantLoginRequest {
    fn parse_data(&self) -> Result<GrantLoginData, &'static str> {
        match serde_json::from_str(&self.data) {
            Ok(v) => Ok(v),
            Err(_) => Err("Invalid data"),
        }
    }
}

#[post("/{product}/combo/granter/login/v2/login")]
pub async fn post_grant_login(
    request: web::Json<GrantLoginRequest>,
    mongo_client: web::Data<MongoClient>,
) -> impl Responder {
    let Ok(data) = request.parse_data() else {
        return r#"{"data":{},"message":"bad request","retcode":5}"#.to_string();
    };
    let Ok(uid) = data.uid.parse::<u32>() else {
        return r#"{"data":{},"message":"bad request","retcode":5}"#.to_string();
    };

    let db = mongo_client.database(DB_NAME);
    let ac_coll = AccountDoc::get_collection(&db);
    let account = match AccountDoc::fetch_by_uid(&ac_coll, uid).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return r#"{"data":{},"message":"account doesn't exist","retcode":1005}"#.to_string();
        }
        Err(e) => {
            tracing::error!("Fetching account by uid: {}", e);
            return r#"{"data":{},"message":"internal error","retcode":2}"#.to_string();
        }
    };

    if &account.token != &data.token {
        return r#"{"data":{},"message":"token mismatch","retcode":1005}"#.to_string();
    }

    let new_token: String;
    match should_refresh_token(&data.token) {
        Ok(true) => {
            new_token = generate_token();
            if let Err(e) = AccountDoc::update_token_by_uid(&ac_coll, uid, &new_token).await {
                tracing::error!("Updating token: {}", e);
                return r#"{"data":{},"message":"internal error","retcode":2}"#.to_string();
            }
        }
        Err(e) => {
            tracing::error!("Refreshing token: {}", e);
            return r#"{"data":{},"message":"internal error","retcode":2}"#.to_string();
        }
        _ => new_token = data.token,
    }

    format!(
        r#"{{"data":{{"account_type":1,"combo_id":"{}","combo_token":"{}","data":"{{\"guest\":false}}","heartbeat":false,"open_id":"{}"}},"message":"OK","retcode":0}}"#,
        account.uid, new_token, account.uid
    )
}

#[derive(Deserialize)]
struct RiskyCheckRequest {
    username: String,
    action_type: String,
}

#[post("/account/risky/api/check")]
pub async fn post_risky_check(
    request: web::Json<RiskyCheckRequest>,
    mongo_client: web::Data<MongoClient>,
) -> impl Responder {
    if &request.action_type != "login" {
        return r#"{"data":{},"message":"OK","retcode":0}"#.to_string();
    }

    let db = mongo_client.database(DB_NAME);
    let ac_coll = AccountDoc::get_collection(&db);
    match AccountDoc::fetch_by_username(&ac_coll, &request.username).await {
        Ok(Some(v)) if v.is_banned => format!(
            r#"{{"data":{{}},"message":"you're banned. reason: {}","retcode":1004}}"#,
            v.ban_reason.unwrap_or_default()
        ),
        Ok(Some(_)) => r#"{"data":{},"message":"OK","retcode":0}"#.to_string(),
        Ok(None) => r#"{"data":{},"message":"account doesn't exist","retcode":1005}"#.to_string(),
        Err(e) => {
            tracing::error!("Fetching account by username: {}", e);
            r#"{"data":{},"message":"internal error","retcode":2}"#.to_string()
        }
    }
}
