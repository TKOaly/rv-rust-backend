use sea_orm::{DatabaseConnection, EntityTrait, FromQueryResult, QuerySelect};
use serde::Serialize;

use crate::{
    db::entities::{prelude::Prodgroup, prodgroup},
    error::Result,
};

#[derive(Serialize, FromQueryResult)]
pub struct Category {
    #[serde(rename = "categoryId")]
    pub id: i32,
    pub description: String,
}

pub async fn get_all_categories(db: &DatabaseConnection) -> Result<Vec<Category>> {
    let categories = Prodgroup::find()
        .select_only()
        .column_as(prodgroup::Column::Pgrpid, "categoryId")
        .column_as(prodgroup::Column::Descr, "description")
        .into_model::<Category>()
        .all(db)
        .await?;

    Ok(categories)
}

pub async fn find_category_by_id(
    category_id: i32,
    db: &DatabaseConnection,
) -> Result<Option<Category>> {
    let category = Prodgroup::find_by_id(category_id)
        .select_only()
        .column_as(prodgroup::Column::Pgrpid, "categoryId")
        .column_as(prodgroup::Column::Descr, "description")
        .into_model::<Category>()
        .one(db)
        .await?;

    Ok(category)
}
