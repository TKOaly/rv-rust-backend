use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromQueryResult, QueryFilter,
    QuerySelect,
};

use crate::{
    db::{
        category::Category,
        entities::{
            prelude::{Price, Prodgroup, Rvitem},
            price, prodgroup, rvitem,
        },
    },
    error::Result,
};

#[derive(Clone, FromQueryResult)]
struct ProductRow {
    barcode: String,
    descr: String,
    pgrpid: Option<i32>,
    pgrdescr: Option<String>,
    count: i32,
    buyprice: i32,
    sellprice: i32,
}

pub struct Product {
    pub barcode: String,
    pub name: String,
    pub category: Category,
    pub buy_price: i32,
    pub sell_price: i32,
    pub stock: i32,
}

impl From<ProductRow> for Product {
    fn from(row: ProductRow) -> Self {
        Self {
            barcode: row.barcode,
            name: row.descr,
            category: Category {
                id: row.pgrpid.unwrap_or(0),
                description: row
                    .pgrdescr
                    .unwrap_or("DEFAULT GROUP, NO DEFINITION".to_string()),
            },
            buy_price: row.buyprice,
            sell_price: row.sellprice,
            stock: row.count,
        }
    }
}

pub async fn search_products(query: &str, db: &DatabaseConnection) -> Result<Vec<Product>> {
    let like = format!("%{}%", query);

    let rows = Rvitem::find()
        .right_join(Price)
        .left_join(Prodgroup)
        .column(rvitem::Column::Descr)
        .column(rvitem::Column::Pgrpid)
        .column_as(prodgroup::Column::Descr, "pgrdescr")
        .column(price::Column::Barcode)
        .column(price::Column::Buyprice)
        .column(price::Column::Sellprice)
        .column(price::Column::Count)
        .filter(
            Condition::all().add(price::Column::Endtime.is_null()).add(
                Condition::any()
                    .add(rvitem::Column::Descr.contains(&like))
                    .add(price::Column::Barcode.contains(&like)),
            ),
        )
        .into_model::<ProductRow>()
        .all(db)
        .await?;

    Ok(rows.iter().map(|r| Product::from(r.clone())).collect())
}
