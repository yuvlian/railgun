use common::proto::GateServer;
use common::proto::prost::Message;
use database::hotfix::HotfixDoc;
use reqwest::Client;

const PROXY_HOST: &str = "proxy1.neonteam.dev";
const CN_PROD_HOST: &str = "prod-gf-cn-dp01.bhsr.com";
const CN_BETA_HOST: &str = "beta-release01-cn.bhsr.com";
const OS_PROD_HOST: &str = "prod-official-asia-dp01.starrails.com";
const OS_BETA_HOST: &str = "beta-release01-asia.starrails.com";

pub async fn fetch_hotfix_from_official(
    reqwest_client: &Client,
    version: &str,
    dispatch_seed: &str,
) -> Result<HotfixDoc, &'static str> {
    let host = match version {
        v if v.starts_with("CNPROD") => CN_PROD_HOST,
        v if v.starts_with("CNBETA") => CN_BETA_HOST,
        v if v.starts_with("OSPROD") => OS_PROD_HOST,
        v if v.starts_with("OSBETA") => OS_BETA_HOST,
        _ => return Err("Invalid version."),
    };

    let target = format!(
        "https://{}/{}/query_gateway?version={}&dispatch_seed={}&language_type=1&platform_type=2&channel_id=1&sub_channel_id=1&is_need_url=1&account_type=1",
        PROXY_HOST, host, version, dispatch_seed
    );

    let reqwest_rsp = match reqwest_client.get(&target).send().await {
        Ok(v) => v.text().await.unwrap(),
        Err(_) => return Err("Reqwest error. Game version is probably not supported."),
    };

    let Ok(base64_decoded) = rbase64::decode(&reqwest_rsp) else {
        return Err("Response wasn't valid base64");
    };
    let Ok(gateserver) = GateServer::decode(base64_decoded.as_ref()) else {
        return Err("Failed decoding GateServer.");
    };

    if gateserver.asset_bundle_url.is_empty() && gateserver.ex_resource_url.is_empty() {
        return Err("AssetBundleUrl and ExResourceUrl is empty.");
    }

    let hotfix_doc = HotfixDoc {
        version: version.to_string(),
        ifix_url: gateserver.ifix_url,
        ifix_version: gateserver.ifix_version,
        mdk_res_url: gateserver.lua_url,
        mdk_res_version: gateserver.mdk_res_version,
        asset_bundle_url: gateserver.asset_bundle_url,
        ex_resource_url: gateserver.ex_resource_url,
    };

    Ok(hotfix_doc)
}
