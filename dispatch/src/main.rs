use actix_web::{App, HttpServer, middleware, web};
use common::server_config::DISPATCH_BIND_TARGET;
use database::MongoClient;
use reqwest::Client;

mod handler;
mod util;

use handler::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let mongo_client: MongoClient = database::new_mongo_client().await;
    common::init_tracing();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::from_fn(util::logging::logger_middleware))
            .app_data(web::Data::new(mongo_client.clone()))
            .app_data(web::Data::new(Client::new()))
            .service(dispatch::get_query_gateway)
            .service(dispatch::get_query_dispatch)
            .service(login::post_login_by_password)
            .service(login::post_login_by_token)
            .service(login::post_grant_login)
            .service(login::post_risky_check)
            .service(registration::get_register)
            .service(registration::post_register)
    })
    .bind(DISPATCH_BIND_TARGET)?
    .run()
    .await
}
