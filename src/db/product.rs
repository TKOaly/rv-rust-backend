use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    FromQueryResult, QueryFilter, QueryOrder, QuerySelect, QueryTrait, TransactionTrait,
};
use serde::Serialize;

use crate::{
    db::{
        actions::Actions,
        category::Category,
        entities::{
            itemhistory,
            prelude::{Itemhistory, Price, Prodgroup, Rvitem, Rvperson, Saldohistory},
            price, prodgroup, rvitem, rvperson, saldohistory,
        },
    },
    error::{AppError, Result},
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

#[derive(Serialize)]
pub struct Product {
    pub barcode: String,
    pub name: String,
    pub category: Category,
    #[serde(rename = "buyPrice")]
    pub buy_price: i32,
    #[serde(rename = "sellPrice")]
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

pub async fn get_products(db: &DatabaseConnection) -> Result<Vec<Product>> {
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
        .filter(Condition::all().add(price::Column::Endtime.is_null()))
        .into_model::<ProductRow>()
        .all(db)
        .await?;

    Ok(rows.iter().map(|r| Product::from(r.clone())).collect())
}

pub async fn find_by_barcode(barcode: &str, db: &DatabaseConnection) -> Result<Product> {
    let row = Rvitem::find()
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
            Condition::all()
                .add(price::Column::Barcode.eq(barcode))
                .add(price::Column::Endtime.is_null()),
        )
        .into_model::<ProductRow>()
        .one(db)
        .await?
        .ok_or(AppError::Internal(anyhow::format_err!(
            "Cannot product by barcode {} from database",
            barcode
        )))?;

    Ok(row.into())
}

pub async fn return_purchase(barcode: &str, user_id: i32, db: &DatabaseConnection) -> Result<bool> {
    let trx = db.begin().await?;
    let now = Utc::now();

    let price = Price::find()
        .filter(price::Column::Barcode.eq(barcode))
        .filter(price::Column::Endtime.is_null())
        .one(&trx)
        .await?;

    let price = match price {
        None => return Ok(false),
        Some(m) => m,
    };

    let five_minutes_ago = now - chrono::Duration::minutes(5);
    let recent_purchase = Itemhistory::find()
        .inner_join(Saldohistory)
        .filter(
            Condition::all()
                .add(itemhistory::Column::Actionid.eq(i32::from(Actions::BoughtBy)))
                .add(itemhistory::Column::Userid.eq(user_id))
                .add(itemhistory::Column::Itemid.eq(price.itemid))
                .add(itemhistory::Column::Time.gt(five_minutes_ago))
                .add(
                    itemhistory::Column::Itemhistid.not_in_subquery(
                        Itemhistory::find()
                            .select_only()
                            .column(itemhistory::Column::Itemhistid2)
                            .filter(itemhistory::Column::Itemhistid2.is_not_null())
                            .into_query(),
                    ),
                ),
        )
        .order_by_desc(itemhistory::Column::Time)
        .one(&trx)
        .await?;

    let recent_purchase = match recent_purchase {
        None => return Ok(false),
        Some(r) => r,
    };

    let saldo_history = Saldohistory::find_by_id(recent_purchase.saldhistid.ok_or_else(|| {
        AppError::Internal(anyhow::format_err!(
            "Column saldhistid missing in query".to_string()
        ))
    })?)
    .one(&trx)
    .await?
    .ok_or_else(|| {
        AppError::Internal(anyhow::format_err!(format!(
            "Cannot find saldo history by id: {:?}",
            recent_purchase.saldhistid
        )))
    })?;

    let refund = -saldo_history.difference;
    let price_id = price.priceid;
    let product_id = price.itemid;
    let stock_now = price.count + 1;

    let mut active_price: price::ActiveModel = price.into();
    active_price.count = Set(stock_now);
    active_price.update(&trx).await?;

    let person = Rvperson::find()
        .filter(rvperson::Column::Userid.eq(user_id))
        .one(&trx)
        .await?
        .ok_or_else(|| {
            AppError::Internal(anyhow::format_err!(
                "Cannot find user by id {} from database",
                user_id
            ))
        })?;

    let new_balance = person.saldo + refund;
    let mut active_person: rvperson::ActiveModel = person.into();
    active_person.saldo = Set(new_balance);
    active_person.update(&trx).await?;

    let inserted_saldohistory = saldohistory::ActiveModel {
        userid: Set(user_id),
        time: Set(now.into()),
        saldo: Set(Some(new_balance)),
        difference: Set(refund),
        ..Default::default()
    }
    .insert(&trx)
    .await?;

    itemhistory::ActiveModel {
        time: Set(now.into()),
        count: Set(Some(stock_now)),
        actionid: Set(i32::from(Actions::ProductReturned)),
        itemid: Set(product_id),
        userid: Set(user_id),
        priceid1: Set(price_id),
        itemhistid2: Set(Some(recent_purchase.itemhistid)),
        saldhistid: Set(Some(inserted_saldohistory.saldhistid)),
        ..Default::default()
    }
    .insert(&trx)
    .await?;

    trx.commit().await?;
    Ok(true)
}

