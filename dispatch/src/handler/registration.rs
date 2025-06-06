use crate::util::password::hash_password;
use crate::util::token::generate_token;
use actix_web::{HttpResponse, Responder, get, http::header::ContentType, post, web};
use database::account::{AccountDoc, AccountMetaDoc};
use database::{DB_NAME, MongoClient};
use serde::Deserialize;

const REGISTER_PAGE: &str = r###"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>/account/register</title><style>*{margin:0;padding:0;box-sizing:border-box;}body{font-family:'Segoe UI',Tahoma,Geneva,Verdana,sans-serif;background-color:#1a1a1a;color:#ffffff;display:flex;justify-content:center;align-items:center;min-height:100vh;}.container{background-color:#2d2d2d;padding:2rem;border-radius:8px;box-shadow:0 4px 6px rgba(0,0,0,0.3);width:100%;max-width:400px;}h1{text-align:center;margin-bottom:2rem;color:#ffffff;}.form-group{margin-bottom:1.5rem;}label{display:block;margin-bottom:0.5rem;color:#cccccc;}input[type="text"],input[type="password"]{width:100%;padding:0.75rem;border:1px solid #555;border-radius:4px;background-color:#3a3a3a;color:#ffffff;font-size:1rem;}input[type="text"]:focus,input[type="password"]:focus{outline:none;border-color:#007acc;box-shadow:0 0 0 2px rgba(0,122,204,0.3);}.btn{width:100%;padding:0.75rem;background-color:#007acc;color:white;border:none;border-radius:4px;font-size:1rem;cursor:pointer;transition:background-color 0.3s;}.btn:hover{background-color:#005a9e;}.btn:disabled{background-color:#555;cursor:not-allowed;}.message{margin-top:1rem;padding:0.75rem;border-radius:4px;text-align:center;display:none;}.message.success{background-color:#2d5a2d;color:#90ee90;border:1px solid #4a8f4a;}.message.error{background-color:#5a2d2d;color:#ff9090;border:1px solid #8f4a4a;}</style></head><body><div class="container"><h1>railgun</h1><form id="registerForm"><div class="form-group"><label for="username">Username</label><input type="text" id="username" name="username" required></div><div class="form-group"><label for="password">Password</label><input type="password" id="password" name="password" required></div><button type="submit" class="btn" id="submitBtn">Register</button></form><div id="message" class="message"></div></div><script>document.getElementById('registerForm').addEventListener('submit',async function(e){e.preventDefault();const submitBtn=document.getElementById('submitBtn');const messageDiv=document.getElementById('message');const username=document.getElementById('username').value;const password=document.getElementById('password').value;submitBtn.disabled=true;submitBtn.textContent='Registering...';messageDiv.style.display='none';try{const response=await fetch('/account/register',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({username:username,password:password})});const responseText=await response.text();messageDiv.textContent=responseText;messageDiv.className=response.ok?'message success':'message error';messageDiv.style.display='block';if(response.ok){document.getElementById('registerForm').reset();}}catch(error){messageDiv.textContent='Network error: '+error.message;messageDiv.className='message error';messageDiv.style.display='block';}finally{submitBtn.disabled=false;submitBtn.textContent='Register';}});</script></body></html>"###;

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

    HttpResponse::Ok().body(format!("Register success! Your uid is {}", uid))
}
