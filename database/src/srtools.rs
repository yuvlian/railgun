use common::time::get_duration_since_unix;
use mongodb::{
    Collection, Database,
    bson::{self, doc},
    error::Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SRTOOLS_COLL_NAME: &str = "srtools";
const SRTOOLS_META_COLL_NAME: &str = "srtools_meta";
const SRTOOLS_SYNC_COOLDOWN_MINUTES: u64 = 30;
const SRTOOLS_EXPORT_COOLDOWN_MINUTES: u64 = 15;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SRToolsData {
    pub avatars: HashMap<String, Avatar>,
    pub relics: Vec<Relic>,
    pub lightcones: Vec<Lightcone>,
    pub battle_config: BattleConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Avatar {
    pub avatar_id: u32,
    pub data: AvatarData,
    pub level: u32,
    pub promotion: u32,
    pub sp_max: u32,
    pub sp_value: u32,
    pub techniques: Vec<u32>,
    pub owner_uid: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AvatarData {
    pub rank: u32,
    pub skills: HashMap<String, u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BattleConfig {
    pub battle_type: String,
    // pub blessings: Vec<Option<serde_json::Value>>,
    // pub custom_stats: Vec<Option<serde_json::Value>>,
    pub cycle_count: u32,
    pub stage_id: u32,
    pub path_resonance_id: u32,
    pub monsters: Vec<Vec<Monster>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Monster {
    pub amount: u32,
    pub level: u32,
    pub monster_id: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Lightcone {
    pub equip_avatar: u32,
    pub internal_uid: u32,
    pub item_id: u32,
    pub level: u32,
    pub promotion: u32,
    pub rank: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Relic {
    pub equip_avatar: u32,
    pub internal_uid: u32,
    pub level: u32,
    pub main_affix_id: u32,
    pub relic_id: u32,
    pub relic_set_id: u32,
    pub sub_affixes: Vec<SubAffix>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubAffix {
    pub count: u32,
    pub step: u32,
    pub sub_affix_id: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SRToolsDoc {
    #[serde(rename = "_id")]
    pub uid: u32,
    pub username: String,
    pub data: Option<SRToolsData>,
}

impl SRToolsDoc {
    pub fn get_collection(db: &Database) -> Collection<Self> {
        db.collection::<Self>(SRTOOLS_COLL_NAME)
    }

    pub async fn insert_to_collection(&self, collection: &Collection<Self>) -> Result<()> {
        collection.insert_one(self).await?;
        Ok(())
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

    pub async fn set_srtools_by_uid(
        collection: &Collection<Self>,
        uid: u32,
        srtools_data: &SRToolsData,
    ) -> Result<()> {
        let srtools_data = bson::to_document(srtools_data).unwrap();
        let filter = doc! { "_id": uid };
        let update = doc! { "$set": { "data": srtools_data }};
        collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn set_srtools_by_username(
        collection: &Collection<Self>,
        username: &str,
        srtools_data: &SRToolsData,
    ) -> Result<()> {
        let srtools_data = bson::to_document(srtools_data).unwrap();
        let filter = doc! { "username": username };
        let update = doc! { "$set": { "data": srtools_data }};
        collection.update_one(filter, update).await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SRToolsMetaDoc {
    #[serde(rename = "_id")]
    pub username: String,
    pub next_sync_allowed: u32,
    pub next_export_allowed: u32,
}

impl SRToolsMetaDoc {
    pub fn get_collection(db: &Database) -> Collection<Self> {
        db.collection::<Self>(SRTOOLS_META_COLL_NAME)
    }

    pub async fn insert_to_collection(&self, collection: &Collection<Self>) -> Result<()> {
        collection.insert_one(self).await?;
        Ok(())
    }

    pub async fn fetch_by_username(
        collection: &Collection<Self>,
        username: &str,
    ) -> Result<Option<Self>> {
        let filter = doc! { "_id": username };
        let result = collection.find_one(filter).await?;
        Ok(result)
    }

    pub async fn update_next_sync_for_username(
        collection: &Collection<Self>,
        username: &str,
    ) -> Result<()> {
        let next_time = (get_duration_since_unix().as_secs() / 60) + SRTOOLS_SYNC_COOLDOWN_MINUTES;
        let next_time = next_time as u32;
        let filter = doc! { "_id": username };
        let update = doc! { "$set": { "next_sync_allowed": next_time } };
        collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn update_next_export_for_username(
        collection: &Collection<Self>,
        username: &str,
    ) -> Result<()> {
        let next_time =
            (get_duration_since_unix().as_secs() / 60) + SRTOOLS_EXPORT_COOLDOWN_MINUTES;
        let next_time = next_time as u32;
        let filter = doc! { "_id": username };
        let update = doc! { "$set": { "next_export_allowed": next_time } };
        collection.update_one(filter, update).await?;
        Ok(())
    }
}
