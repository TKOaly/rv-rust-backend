use crate::{
    db::entities::{
        prelude::{Rvperson, TempPassword},
        temppassword,
    },
    error::{AppError, Result},
};
use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;

#[derive(Debug, Serialize, Eq, PartialEq)]
pub enum Role {
    Admin,
    User,
    Inactive,
}

impl From<i32> for Role {
    fn from(id: i32) -> Self {
        match id {
            1 => Role::Admin,
            7 => Role::Inactive,
            _ => Role::User,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PrivacyLevel {
    NoLimits,
    HideUsername,
    HideAll,
}

impl From<i32> for PrivacyLevel {
    fn from(privacy_level: i32) -> Self {
        match privacy_level {
            0 => PrivacyLevel::NoLimits,
            1 => PrivacyLevel::HideUsername,
            2 => PrivacyLevel::HideAll,
            _ => PrivacyLevel::NoLimits,
        }
    }
}

pub struct User {
    pub id: i32,
    pub username: String,
    pub realname: String,
    pub email: String,
    pub role: Role,
    pub saldo: i32,
    pub privacy_level: PrivacyLevel,
    pub password_hash: String,
    pub temp_password_hash: Option<String>,
    pub rfid_hash: Option<String>,
}

pub async fn get_user_by_user_id(user_id: i32, db: &DatabaseConnection) -> Result<User> {
    let cutoff = Utc::now() - Duration::minutes(15);

    let (user, temp_password) = Rvperson::find_by_id(user_id)
        .find_also_related(TempPassword)
        .filter(
            Condition::any()
                .add(temppassword::Column::CreatedAt.gte(cutoff))
                .add(temppassword::Column::Userid.is_null()),
        )
        .one(db)
        .await?
        .ok_or(AppError::Internal(anyhow::format_err!(
            "Cannot find user by id {}",
            user_id
        )))?;

    Ok(User {
        id: user.userid,
        username: user.name,
        realname: user.realname.unwrap_or_default(),
        email: user.univident,
        role: user.roleid.into(),
        saldo: user.saldo,
        privacy_level: user.privacy_level.into(),
        password_hash: user.pass,
        rfid_hash: user.rfid,
        temp_password_hash: temp_password.map(|m| m.temp_password),
    })
}
