use chrono::{DateTime, Utc};
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

#[derive(Serialize)]
pub struct PurchaseEvent {
    pub id: i32,
    pub time: DateTime<Utc>,
    pub price: i32,
    #[serde(rename = "balanceAfter")]
    pub balance_after: i32,
    #[serde(rename = "stockAfter")]
    pub stock_after: i32,
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

// pub async fn insert_product(
//     product_data: InsertProductData,
//     db: &DatabaseConnection,
//     user_id: i32,
// ) -> Result<Product> {
//     let txn = db.begin().await?;
//     let now = Utc::now().naive_utc();

//     let inserted_item = rvitem::ActiveModel {
//         pgrpid: Set(Some(product_data.category_id)),
//         descr: Set(product_data.name.clone()),
//         ..Default::default()
//     }
//     .insert(&txn)
//     .await?;

//     let inserted_price = price::ActiveModel {
//         barcode: Set(product_data.barcode.clone()),
//         count: Set(product_data.stock),
//         buyprice: Set(product_data.buy_price),
//         sellprice: Set(product_data.sell_price),
//         itemid: Set(inserted_item.itemid),
//         userid: Set(Some(user_id)),
//         starttime: Set(Some(now)),
//         endtime: Set(None),
//         ..Default::default()
//     }
//     .insert(&txn)
//     .await?;

//     let category_descr = prodgroup::Entity::find_by_id(product_data.category_id)
//         .one(&txn)
//         .await?
//         .map(|r| r.descr)
//         .unwrap_or_default();

//     itemhistory::ActiveModel {
//         time: Set(Some(now)),
//         count: Set(product_data.stock),
//         actionid: Set(Some(actions::ITEM_CREATED)),
//         userid: Set(Some(user_id)),
//         itemid: Set(Some(inserted_item.itemid)),
//         priceid1: Set(Some(inserted_price.priceid)),
//         ..Default::default()
//     }
//     .insert(&txn)
//     .await?;

//     txn.commit().await?;

//     Ok(Product {
//         barcode: product_data.barcode,
//         name: product_data.name,
//         category: Category {
//             category_id: product_data.category_id,
//             description: category_descr,
//         },
//         buy_price: product_data.buy_price,
//         sell_price: product_data.sell_price,
//         stock: product_data.stock,
//     })
// }

// pub async fn update_product(
//     barcode: &str,
//     db: &DatabaseConnection,
//     product_data: UpdateProductData,
//     user_id: i32,
// ) -> Result<Option<Product>> {
//     let txn = db.begin().await?;

//     if product_data.name.is_some() || product_data.category_id.is_some() {
//         let price_row = price::Entity::find()
//             .filter(price::Column::Barcode.eq(barcode))
//             .filter(price::Column::Endtime.is_null())
//             .one(&txn)
//             .await?;

//         if let Some(price_row) = price_row {
//             if let Some(item) = price_row.find_related(rvitem::Entity).one(&txn).await? {
//                 let mut active: rvitem::ActiveModel = item.into();
//                 if let Some(name) = product_data.name {
//                     active.descr = Set(name);
//                 }
//                 if let Some(cat_id) = product_data.category_id {
//                     active.pgrpid = Set(Some(cat_id));
//                 }
//                 active.update(&txn).await?;
//             }
//         }
//     }

//     if product_data.stock.is_some() || product_data.buy_price.is_some() || product_data.sell_price.is_some() {
//         let current = price::Entity::find()
//             .filter(price::Column::Barcode.eq(barcode))
//             .filter(price::Column::Endtime.is_null())
//             .one(&txn)
//             .await?;

//         if let Some(current) = current {
//             if product_data.sell_price.is_none() {
//                 let mut active: price::ActiveModel = current.into();
//                 if let Some(stock) = product_data.stock {
//                     active.count = Set(Some(stock));
//                 }
//                 if let Some(buy_price) = product_data.buy_price {
//                     active.buyprice = Set(Some(buy_price));
//                 }
//                 active.update(&txn).await?;
//             } else {
//                 let now = Utc::now().naive_utc();
//                 let mut end_active: price::ActiveModel = current.clone().into();
//                 end_active.endtime = Set(Some(now));
//                 end_active.update(&txn).await?;

//                 price::ActiveModel {
//                     barcode: Set(current.barcode),
//                     count: Set(product_data.stock.map(Some).unwrap_or(current.count)),
//                     buyprice: Set(product_data.buy_price.map(Some).unwrap_or(current.buyprice)),
//                     sellprice: Set(Some(product_data.sell_price.unwrap())),
//                     itemid: Set(current.itemid),
//                     userid: Set(Some(user_id)),
//                     starttime: Set(Some(now)),
//                     endtime: Set(None),
//                     ..Default::default()
//                 }
//                 .insert(&txn)
//                 .await?;
//             }
//         }
//     }

//     let updated_price = price::Entity::find()
//         .filter(price::Column::Barcode.eq(barcode))
//         .filter(price::Column::Endtime.is_null())
//         .one(&txn)
//         .await?;

//     txn.commit().await?;

//     match updated_price {
//         None => Ok(None),
//         Some(price_row) => {
//             let item = price_row.find_related(rvitem::Entity).one(db).await?;
//             match item {
//                 None => Ok(None),
//                 Some(item) => {
//                     let group = item.find_related(prodgroup::Entity).one(db).await?;
//                     Ok(Some(Product::from((price_row.clone(), item, group))))
//                 }
//             }
//         }
//     }
// }

pub async fn record_purchase(
    barcode: &str,
    user_id: i32,
    count: i32,
    db: &DatabaseConnection,
) -> Result<Vec<PurchaseEvent>> {
    let trx = db.begin().await?;
    let now = Utc::now();

    let price = Price::find()
        .filter(price::Column::Barcode.eq(barcode))
        .filter(price::Column::Endtime.is_null())
        .one(&trx)
        .await?
        .ok_or_else(|| {
            AppError::Internal(anyhow::format_err!(
                "Cannot find price by barcode: {}",
                barcode
            ))
        })?;

    let stock_before = price.count;
    let sell_price = price.sellprice;
    let price_id = price.priceid;
    let item_id = price.itemid;

    let mut active_price: price::ActiveModel = price.into();
    active_price.count = Set(stock_before - count);
    active_price.update(&trx).await?;

    let user = Rvperson::find()
        .filter(rvperson::Column::Userid.eq(user_id))
        .one(&trx)
        .await?
        .ok_or_else(|| {
            AppError::Internal(anyhow::format_err!(
                "Cannot find user by user_id {}",
                user_id
            ))
        })?;

    let balance_before = user.saldo;
    let mut active_user: rvperson::ActiveModel = user.into();
    active_user.saldo = Set(balance_before - count * sell_price);
    active_user.update(&trx).await?;

    let mut stock = stock_before;
    let mut balance = balance_before;
    let mut purchases = Vec::new();

    /* Storing multibuy into history as multiple individual history events. */
    for _ in 0..count {
        stock -= 1;
        balance -= sell_price;

        let saldo_history = saldohistory::ActiveModel {
            userid: Set(user_id),
            time: Set(now.into()),
            saldo: Set(Some(balance)),
            difference: Set(-sell_price),
            ..Default::default()
        }
        .insert(&trx)
        .await?;

        let item_history = itemhistory::ActiveModel {
            time: Set(now.into()),
            count: Set(Some(stock)),
            actionid: Set(i32::from(Actions::BoughtBy)),
            itemid: Set(item_id),
            userid: Set(user_id),
            priceid1: Set(price_id),
            saldhistid: Set(Some(saldo_history.saldhistid)),
            ..Default::default()
        }
        .insert(&trx)
        .await?;

        purchases.push(PurchaseEvent {
            id: item_history.itemhistid,
            time: now,
            price: sell_price,
            balance_after: balance,
            stock_after: stock,
        });
    }

    trx.commit().await?;
    Ok(purchases)
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

