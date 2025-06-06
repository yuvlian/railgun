use mongodb::{Collection, Database, bson::doc, error::Result};
use serde::{Deserialize, Serialize};

const HOTFIX_COLL_NAME: &str = "hotfix";

#[derive(Debug, Deserialize, Serialize)]
pub struct HotfixDoc {
    #[serde(rename = "_id")]
    pub version: String,
    pub ifix_url: String,
    pub ifix_version: String,
    pub mdk_res_url: String,
    pub mdk_res_version: String,
    pub asset_bundle_url: String,
    pub ex_resource_url: String,
}

impl HotfixDoc {
    pub fn get_collection(db: &Database) -> Collection<Self> {
        db.collection::<Self>(HOTFIX_COLL_NAME)
    }

    pub async fn insert_to_collection(&self, collection: &Collection<Self>) -> Result<()> {
        collection.insert_one(self).await?;
        Ok(())
    }

    pub async fn fetch_by_version(
        collection: &Collection<Self>,
        version: &str,
    ) -> Result<Option<Self>> {
        let filter = doc! { "_id": version };
        let result = collection.find_one(filter).await?;
        Ok(result)
    }

    pub async fn replace_in_collection(&self, collection: &Collection<Self>) -> Result<()> {
        let filter = doc! { "_id": &self.version };
        collection.replace_one(filter, self).await?;
        Ok(())
    }
}
