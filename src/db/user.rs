use crate::{
    config::AppConfig,
    db::entities::{
        prelude::{Rvperson, TempPassword},
        rvperson, temppassword,
    },
    error::{AppError, Result},
};
use bcrypt;
use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use sha2::{Digest, Sha256};

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

pub async fn find_user_by_id(user_id: i32, db: &DatabaseConnection) -> Result<User> {
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
            "Cannot find user by id {} from database",
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

pub async fn find_user_by_username(username: &str, db: &DatabaseConnection) -> Result<User> {
    let cutoff = Utc::now() - Duration::minutes(15);

    let (user, temp_password) = Rvperson::find()
        .find_also_related(TempPassword)
        .filter(
            Condition::all()
                .add(rvperson::Column::Name.eq(username))
                .add(
                    Condition::any()
                        .add(temppassword::Column::CreatedAt.gte(cutoff))
                        .add(temppassword::Column::Userid.is_null()),
                ),
        )
        .one(db)
        .await?
        .ok_or(AppError::Internal(anyhow::format_err!(
            "Cannot find user by username {} from database",
            username
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

fn old_rfid_hash(rfid_hex: &str) -> Result<String> {
    let rfid_bytes = hex::decode(rfid_hex)?;
    let salt_bytes = "rv-vakio-suola".as_bytes();
    let mut hasher = Sha256::new();
    hasher.update(salt_bytes);
    hasher.update(rfid_bytes);
    let digest = hasher.finalize();
    let hex_str = hex::encode(digest);

    let filtered = hex_str
        .chars()
        .enumerate()
        .filter(|(idx, c)| !(idx % 2 == 0 && *c == '0'))
        .map(|(_, c)| c)
        .collect();

    Ok(filtered)
}

fn new_rfid_hash(rfid_hex: &str, config: &AppConfig) -> Result<String> {
    let hash = bcrypt::hash_with_salt(rfid_hex, 11, config.rfid_salt)?;
    Ok(hash.format_for_version(bcrypt::Version::TwoB))
}

async fn migrate_rfid_hash(rfid: &str, db: &DatabaseConnection) -> Result<User> {
    let cutoff = Utc::now() - Duration::minutes(15);
    let rfid_hash = old_rfid_hash(rfid)?;

    let user = Rvperson::find()
        .find_also_related(TempPassword)
        .filter(
            Condition::all()
                .add(rvperson::Column::Rfid.eq(rfid_hash))
                .add(
                    Condition::any()
                        .add(temppassword::Column::CreatedAt.gte(cutoff))
                        .add(temppassword::Column::Userid.is_null()),
                ),
        )
        .one(db)
        .await?;

    match user {
        Some((user, temp_password)) => {
            tracing::info!("'Migrated user: {} rfid has to use bcrypt", user.name);
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
        None => Err(AppError::NotFound),
    }
}

pub async fn find_by_rfid(rfid: &str, config: &AppConfig, db: &DatabaseConnection) -> Result<User> {
    let cutoff = Utc::now() - Duration::minutes(15);
    let rfid_hash = new_rfid_hash(rfid, config)?;

    let user = Rvperson::find()
        .find_also_related(TempPassword)
        .filter(
            Condition::all()
                .add(rvperson::Column::Rfid.eq(rfid_hash))
                .add(
                    Condition::any()
                        .add(temppassword::Column::CreatedAt.gte(cutoff))
                        .add(temppassword::Column::Userid.is_null()),
                ),
        )
        .one(db)
        .await?;

    match user {
        Some((user, temp_password)) => Ok(User {
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
        }),
        None => migrate_rfid_hash(rfid, db).await,
    }
}
