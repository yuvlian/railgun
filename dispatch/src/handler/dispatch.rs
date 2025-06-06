use crate::util::auto_hotfix;
use actix_web::{Responder, get, web};
use common::proto::prost::Message;
use common::proto::{Dispatch, GateServer, RegionInfo};
use common::server_config::{
    DISPATCH_ALWAYS_CHECK_HOTFIX, DISPATCH_BIND_TARGET, DISPATCH_ENV_TYPE, DISPATCH_REGION_NAME,
    GAMESERVER_BIND_TARGET,
};
use database::hotfix::HotfixDoc;
use database::{DB_NAME, MongoClient};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct GatewayQuery {
    version: String,
    dispatch_seed: String,
}

#[get("/query_gateway")]
pub async fn get_query_gateway(
    query: web::Query<GatewayQuery>,
    reqwest_client: web::Data<Client>,
    mongo_client: web::Data<MongoClient>,
) -> impl Responder {
    let db = mongo_client.database(DB_NAME);
    let hf_coll = HotfixDoc::get_collection(&db);
    let hf = match HotfixDoc::fetch_by_version(&hf_coll, &query.version).await {
        Ok(Some(v)) if !DISPATCH_ALWAYS_CHECK_HOTFIX => v,
        _ => {
            let hf2 = match auto_hotfix::fetch_hotfix_from_official(
                &reqwest_client,
                &query.version,
                &query.dispatch_seed,
            )
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("AutoHotfix: {}", e);
                    HotfixDoc {
                        version: query.version.clone(),
                        ifix_url: String::new(),
                        ifix_version: String::from("0"),
                        mdk_res_url: String::new(),
                        mdk_res_version: String::new(),
                        asset_bundle_url: String::new(),
                        ex_resource_url: String::new(),
                    }
                }
            };
            if let Err(e) = hf2.insert_to_collection(&hf_coll).await {
                tracing::error!("Inserting HotfixDoc: {}", e);
            }
            hf2
        }
    };

    let gateserver = GateServer {
        use_tcp: true,
        ip: GAMESERVER_BIND_TARGET.0.to_string(),
        port: GAMESERVER_BIND_TARGET.1 as u32,

        lua_url: hf.mdk_res_url,
        ifix_url: hf.ifix_url,
        ex_resource_url: hf.ex_resource_url,
        asset_bundle_url: hf.asset_bundle_url,
        mdk_res_version: hf.mdk_res_version,
        ifix_version: hf.ifix_version,

        enable_version_update: true,
        enable_design_data_version_update: true,
        enable_save_replay_file: true,
        enable_upload_battle_log: true,
        enable_watermark: true,
        event_tracking_open: true,
        ..Default::default()
    }
    .encode_to_vec();

    rbase64::encode(&gateserver)
}

#[get("/query_dispatch")]
pub async fn get_query_dispatch() -> impl Responder {
    let dispatch = Dispatch {
        region_list: vec![RegionInfo {
            name: DISPATCH_REGION_NAME.to_string(),
            title: DISPATCH_REGION_NAME.to_string(),
            env_type: DISPATCH_ENV_TYPE.to_string(),
            dispatch_url: format!(
                "http://{}:{}/query_gateway",
                DISPATCH_BIND_TARGET.0, DISPATCH_BIND_TARGET.1
            ),
            ..Default::default()
        }],
        ..Default::default()
    }
    .encode_to_vec();

    rbase64::encode(&dispatch)
}
