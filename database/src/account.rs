use mongodb::{Collection, Database, bson::doc, error::Result};
use serde::{Deserialize, Serialize};

const ACCOUNT_COLL_NAME: &str = "account";
const ACCOUNT_META_COLL_NAME: &str = "account_meta";
const STARTING_UID: u32 = 10000;

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountDoc {
    #[serde(rename = "_id")]
    pub uid: u32,
    pub username: String,
    pub password_hash: String,
    pub token: String,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
}

impl AccountDoc {
    pub fn get_collection(db: &Database) -> Collection<Self> {
        db.collection::<Self>(ACCOUNT_COLL_NAME)
    }

    pub async fn insert_to_collection(&self, collection: &Collection<Self>) -> Result<()> {
        collection.insert_one(self).await?;
        Ok(())
    }

    pub async fn check_username_taken(
        collection: &Collection<Self>,
        username: &str,
    ) -> Result<bool> {
        let filter = doc! { "username": username };
        let result = collection.find_one(filter).await?;
        Ok(result.is_some())
    }

    pub async fn fetch_by_uid(collection: &Collection<Self>, uid: u32) -> Result<Option<Self>> {
        let filter = doc! { "_id": uid };
        let result = collection.find_one(filter).await?;
        Ok(result)
    }

    pub async fn fetch_by_username(
        collection: &Collection<Self>,
        username: &str,
    ) -> Result<Option<Self>> {
        let filter = doc! { "username": username };
        let result = collection.find_one(filter).await?;
        Ok(result)
    }

    pub async fn update_token_by_uid(
        collection: &Collection<Self>,
        uid: u32,
        new_token: &str,
    ) -> Result<()> {
        let filter = doc! { "_id": uid };
        let update = doc! { "$set": { "token": new_token } };
        collection.update_one(filter, update).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountMetaDoc {
    #[serde(rename = "_id")]
    pub id: String,
    pub next_uid: u32,
}

impl AccountMetaDoc {
    pub fn get_collection(db: &Database) -> Collection<Self> {
        db.collection::<Self>(ACCOUNT_META_COLL_NAME)
    }

    pub async fn get_next_uid(collection: &Collection<Self>) -> Result<u32> {
        let filter = doc! { "_id": "meta" };
        let update = doc! { "$inc": { "next_uid": 1 } };
        let result = collection.find_one_and_update(filter, update).await?;
        if let Some(doc) = result {
            Ok(doc.next_uid)
        } else {
            let next_uid = Self::init_account_meta(collection).await?;
            Ok(next_uid)
        }
    }

    async fn init_account_meta(collection: &Collection<Self>) -> Result<u32> {
        let default = Self {
            id: String::from("meta"),
            next_uid: STARTING_UID + 1,
        };
        collection.insert_one(&default).await?;
        Ok(STARTING_UID)
    }
}
