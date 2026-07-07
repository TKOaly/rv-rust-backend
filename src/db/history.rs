use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromQueryResult, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select, sea_query::Expr,
};
use serde::Serialize;

use crate::{
    db::{
        actions::Actions,
        category::Category,
        entities::{
            itemhistory, personhist,
            prelude::{Itemhistory, Personhist},
            price, prodgroup, rvitem, rvperson, saldohistory,
        },
        product::Product,
        user::Role,
    },
    error::{AppError, Result},
    routes::user::UserResponse,
};

#[derive(FromQueryResult)]
struct PurchaseHistoryRow {
    purchase_id: i32,
    time: DateTime<Utc>,
    price: i32,
    returned: bool,
    balance_after: i32,
    barcode: String,
    name: String,
    sell_price: i32,
    stock: i32,
    category_id: i32,
    category_description: String,
    userid: i32,
    username: String,
    realname: String,
    univident: String,
    roleid: i32,
    privacylevel: i32,
}

#[derive(FromQueryResult)]
struct DepositHistoryRow {
    pershistid: i32,
    time: DateTime<Utc>,
    difference: i32,
    saldo: i32,
    userid: i32,
    name: String,
    realname: String,
    univident: String,
    current_saldo: i32,
    roleid: i32,
    actionid: i32,
    privacylevel: i32,
}

#[derive(Serialize)]
pub struct DepositHistoryData {
    #[serde(rename = "depositId")]
    pub deposit_id: i32,
    pub time: DateTime<Utc>,
    pub amount: i32,
    #[serde(rename = "balanceAfter")]
    pub balance_after: i32,
    #[serde(rename = "type")]
    pub action_id: i32,
    pub user: UserResponse,
}

impl From<DepositHistoryRow> for DepositHistoryData {
    fn from(row: DepositHistoryRow) -> Self {
        Self {
            deposit_id: row.pershistid,
            time: row.time,
            amount: row.difference,
            balance_after: row.saldo,
            action_id: row.actionid,
            user: UserResponse {
                user_id: row.userid,
                username: row.name,
                full_name: row.realname,
                email: row.univident,
                money_balance: row.current_saldo,
                role: format!("{:?}", Role::from(row.roleid)),
                privacy_level: row.privacylevel,
            },
        }
    }
}

#[derive(Serialize)]
pub struct PurchaseHistoryData {
    pub id: i32,
    pub time: DateTime<Utc>,
    pub price: i32,
    pub returned: bool,
    pub product: Product,
    pub sell_price: i32,
    pub stock: i32,
    pub balance_after: i32,
    pub user: UserResponse,
}

impl From<PurchaseHistoryRow> for PurchaseHistoryData {
    fn from(value: PurchaseHistoryRow) -> Self {
        Self {
            id: value.purchase_id,
            time: value.time,
            price: value.price,
            returned: value.returned,
            product: Product {
                barcode: value.barcode,
                name: value.name,
                category: Category {
                    id: value.category_id,
                    description: value.category_description,
                },
                buy_price: 0,
                sell_price: value.sell_price,
                stock: value.stock,
            },
            sell_price: value.sell_price,
            stock: value.stock,
            balance_after: value.balance_after,
            user: UserResponse {
                user_id: value.userid,
                username: value.username,
                full_name: value.realname,
                email: value.univident,
                money_balance: 0,
                role: format!("{:?}", Role::from(value.roleid)),
                privacy_level: value.privacylevel,
            },
        }
    }
}

fn purchase_history_query() -> Select<Itemhistory> {
    Itemhistory::find()
        .join(JoinType::LeftJoin, itemhistory::Relation::Rvitem.def())
        .join(JoinType::LeftJoin, rvitem::Relation::Prodgroup.def())
        .join(JoinType::LeftJoin, itemhistory::Relation::Price1.def())
        .join(
            JoinType::LeftJoin,
            itemhistory::Relation::Saldohistory.def(),
        )
        .join(JoinType::LeftJoin, itemhistory::Relation::Rvperson.def())
        .join_as(
            JoinType::LeftJoin,
            itemhistory::Relation::SelfRef.def(),
            "ih2",
        )
        .select_only()
        .column(itemhistory::Column::Itemhistid)
        .column(itemhistory::Column::Time)
        .column(itemhistory::Column::Count)
        .column_as(rvitem::Column::Descr, "name")
        .column(rvitem::Column::Pgrpid)
        .column_as(prodgroup::Column::Descr, "description")
        .column(price::Column::Barcode)
        .column(price::Column::Sellprice)
        .column(price::Column::Buyprice)
        .column_as(price::Column::Count, "stock")
        .column(saldohistory::Column::Saldo)
        .column(rvperson::Column::Userid)
        .column_as(rvperson::Column::Name, "username")
        .column(rvperson::Column::Realname)
        .column(rvperson::Column::Univident)
        .column(rvperson::Column::Roleid)
        .column(rvperson::Column::PrivacyLevel)
        .expr_as(Expr::cust("(ih2.itemhistid2 IS NOT NULL)"), "returned")
        .filter(itemhistory::Column::Actionid.eq(i32::from(Actions::BoughtBy)))
        .order_by_desc(itemhistory::Column::Time)
        .order_by_desc(itemhistory::Column::Itemhistid)
}

#[allow(deprecated)] //deprecation allowed because database has all action ids
fn deposit_history_query() -> Select<Personhist> {
    Personhist::find()
        .join(JoinType::LeftJoin, personhist::Relation::Saldohistory.def())
        .join(JoinType::LeftJoin, personhist::Relation::Rvperson1.def())
        .select_only()
        .column(personhist::Column::Pershistid)
        .column(personhist::Column::Time)
        .column(saldohistory::Column::Difference)
        .column(saldohistory::Column::Saldo)
        .column(rvperson::Column::Userid)
        .column(rvperson::Column::Name)
        .column(rvperson::Column::Realname)
        .column(rvperson::Column::Univident)
        .column(rvperson::Column::Roleid)
        .column(rvperson::Column::PrivacyLevel)
        .filter(
            Condition::any()
                .add(personhist::Column::Actionid.eq(i32::from(Actions::DepositedMoneyCash)))
                .add(
                    personhist::Column::Actionid.eq(i32::from(Actions::DepositedMoneyBankTransfer)),
                )
                .add(personhist::Column::Actionid.eq(i32::from(Actions::DepositedMoney))),
        )
        .order_by_desc(personhist::Column::Time)
        .order_by_desc(personhist::Column::Pershistid)
}

pub async fn find_purchase_history(
    offset: Option<i32>,
    limit: Option<u64>,
    db: &DatabaseConnection,
) -> Result<Vec<PurchaseHistoryData>> {
    let mut query = purchase_history_query();
    if let Some(offset) = offset {
        query = query.filter(itemhistory::Column::Itemhistid.lt(offset));
    }
    if let Some(limit) = limit {
        query = query.limit(limit);
    }

    let result: Vec<PurchaseHistoryRow> = query.into_model().all(db).await?;
    Ok(result.into_iter().map(Into::into).collect())
}

pub async fn find_purchase_history_by_user_id(
    user_id: i32,
    db: &DatabaseConnection,
) -> Result<Vec<PurchaseHistoryData>> {
    let result: Vec<PurchaseHistoryRow> = purchase_history_query()
        .filter(itemhistory::Column::Userid.eq(user_id))
        .into_model::<PurchaseHistoryRow>()
        .all(db)
        .await?;

    Ok(result.into_iter().map(Into::into).collect())
}

pub async fn find_purchase_history_by_barcode(
    barcode: String,
    db: &DatabaseConnection,
) -> Result<Vec<PurchaseHistoryData>> {
    let result: Vec<PurchaseHistoryRow> = purchase_history_query()
        .filter(price::Column::Barcode.eq(barcode))
        .into_model::<PurchaseHistoryRow>()
        .all(db)
        .await?;

    Ok(result.into_iter().map(Into::into).collect())
}

pub async fn find_purchase_by_id(
    purchase_id: i32,
    db: &DatabaseConnection,
) -> Result<PurchaseHistoryData> {
    let result = purchase_history_query()
        .filter(itemhistory::Column::Itemhistid.eq(purchase_id))
        .into_model::<PurchaseHistoryRow>()
        .one(db)
        .await?;

    result
        .map(Into::into)
        .ok_or_else(|| AppError::NotFound("Purchase was not found".to_string()))
}

pub async fn find_purchase_history_by_user_id_and_purchase_id(
    user_id: i32,
    purchase_id: i32,
    db: &DatabaseConnection,
) -> Result<PurchaseHistoryData> {
    let result = purchase_history_query()
        .filter(itemhistory::Column::Userid.eq(user_id))
        .filter(itemhistory::Column::Itemhistid.eq(purchase_id))
        .into_model::<PurchaseHistoryRow>()
        .one(db)
        .await?;

    result
        .map(Into::into)
        .ok_or_else(|| AppError::NotFound("Purchase was not found".to_string()))
}

pub async fn find_deposit_history(
    offset: Option<i32>,
    limit: Option<u64>,
    db: &DatabaseConnection,
) -> Result<Vec<DepositHistoryData>> {
    let mut query = deposit_history_query();
    if let Some(offset) = offset {
        query = query.filter(personhist::Column::Pershistid.lt(offset));
    }
    if let Some(limit) = limit {
        query = query.limit(limit);
    }

    let result: Vec<DepositHistoryRow> = query.into_model().all(db).await?;
    Ok(result.into_iter().map(Into::into).collect())
}

pub async fn find_deposit_history_by_user_id(
    user_id: i32,
    db: &DatabaseConnection,
) -> Result<Vec<DepositHistoryData>> {
    let result: Vec<DepositHistoryRow> = deposit_history_query()
        .filter(personhist::Column::Userid1.eq(user_id))
        .into_model::<DepositHistoryRow>()
        .all(db)
        .await?;

    Ok(result.into_iter().map(Into::into).collect())
}

pub async fn find_deposit_history_by_user_id_and_deposit_id(
    user_id: i32,
    deposit_id: i32,
    db: &DatabaseConnection,
) -> Result<DepositHistoryData> {
    let result = deposit_history_query()
        .filter(
            Condition::all()
                .add(personhist::Column::Pershistid.eq(deposit_id))
                .add(personhist::Column::Userid1.eq(user_id)),
        )
        .into_model::<DepositHistoryRow>()
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("Deposit was not found".to_string()))?;

    Ok(result.into())
}

pub async fn find_deposit_by_id(
    deposit_id: i32,
    db: &DatabaseConnection,
) -> Result<DepositHistoryData> {
    let result = deposit_history_query()
        .filter(personhist::Column::Pershistid.eq(deposit_id))
        .into_model::<DepositHistoryRow>()
        .one(db)
        .await?;

    result
        .map(Into::into)
        .ok_or_else(|| AppError::NotFound("Deposit was not found".to_string()))
}
