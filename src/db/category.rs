use sea_orm::FromQueryResult;
use serde::Serialize;

#[derive(Serialize, FromQueryResult)]
pub struct Category {
    #[serde(rename = "categoryId")]
    pub id: i32,
    pub description: String,
}
