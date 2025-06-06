pub use mongodb::Client as MongoClient;

pub mod account;
pub mod hotfix;

const MONGODB_CONNECTION_URI: &str = "mongodb://localhost:27017";
pub const DB_NAME: &str = "railgun";

pub async fn new_mongo_client() -> MongoClient {
    MongoClient::with_uri_str(MONGODB_CONNECTION_URI)
        .await
        .unwrap()
}
