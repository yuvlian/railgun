use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use common::server_config::DISPATCH_BIND_TARGET;
use database::MongoClient;
use reqwest::Client;

mod handler;
mod util;

use handler::*;
use util::certs;

// just adding this to test how long will it compile
use common::resource::ExcelOutput;
use common::resource::LevelOutput;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    {
        let ex = ExcelOutput::get("AchievementData.json");
        let lv = LevelOutput::get("Map/MapInfo_P10000_F10000000.json");
        println!("{} {}", ex.is_some(), lv.is_some());
    }
    common::init_tracing();
    certs::check_cert_exists();

    let ssl_acceptor = certs::tls_builder();
    let mongo_client: MongoClient = database::new_mongo_client().await;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["GET", "HEAD", "OPTIONS", "POST"]);

        App::new()
            .wrap(cors)
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
            .service(srtools::get_json)
            // .service(srtools::options_cors)
            .service(srtools::post_sync)
    })
    .bind_openssl(DISPATCH_BIND_TARGET, ssl_acceptor)?
    .run()
    .await
}
