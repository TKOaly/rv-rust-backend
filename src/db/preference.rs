use sea_orm::{DatabaseConnection, EntityTrait, FromQueryResult, QuerySelect};
use serde::Serialize;

use crate::{
    db::entities::{preferences, prelude::Preferences},
    error::Result,
};

#[derive(Serialize, FromQueryResult)]
pub struct Preference {
    #[serde(rename = "categoryId")]
    pub key: i32,
    pub value: String,
}

pub async fn get_all_preferences(db: &DatabaseConnection) -> Result<Vec<Preference>> {
    let preferences = Preferences::find()
        .select_only()
        .column(preferences::Column::Value)
        .column(preferences::Column::Key)
        .into_model::<Preference>()
        .all(db)
        .await?;

    Ok(preferences)
}

pub async fn find_preference_by_key(
    key: &str,
    db: &DatabaseConnection,
) -> Result<Option<Preference>> {
    let preferences = Preferences::find_by_id(key)
        .select_only()
        .column(preferences::Column::Value)
        .column(preferences::Column::Key)
        .into_model::<Preference>()
        .one(db)
        .await?;

    Ok(preferences)
}
