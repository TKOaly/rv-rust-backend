use sea_orm::FromQueryResult;
use serde::Deserialize;

#[derive(Deserialize, FromQueryResult)]
pub struct Category {
    #[serde(rename = "pgrpid")]
    pub id: i32,
    #[serde(rename = "pgrdescr")]
    pub description: String,
}
