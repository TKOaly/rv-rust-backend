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
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter,
};
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

impl From<Role> for i32 {
    fn from(role: Role) -> Self {
        match role {
            Role::Admin => 1,
            Role::User => 2,
            Role::Inactive => 7,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PrivacyLevel {
    NoLimits,
    HideUsername,
    HideAll,
}

impl From<PrivacyLevel> for i32 {
    fn from(value: PrivacyLevel) -> Self {
        match value {
            PrivacyLevel::NoLimits => 0,
            PrivacyLevel::HideUsername => 1,
            PrivacyLevel::HideAll => 2,
        }
    }
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
    pub full_name: String,
    pub email: String,
    pub role: Role,
    pub saldo: i32,
    pub privacy_level: PrivacyLevel,
    pub password_hash: String,
    pub temp_password_hash: Option<String>,
    pub rfid_hash: Option<String>,
}

impl From<(rvperson::Model, Option<temppassword::Model>)> for User {
    fn from((user, temp_password): (rvperson::Model, Option<temppassword::Model>)) -> Self {
        Self {
            id: user.userid,
            username: user.name,
            full_name: user.realname.unwrap_or("No name".to_string()),
            email: user.univident,
            role: user.roleid.into(),
            saldo: user.saldo,
            privacy_level: user.privacy_level.into(),
            password_hash: user.pass,
            rfid_hash: user.rfid,
            temp_password_hash: temp_password.map(|m| m.temp_password),
        }
    }
}

impl From<rvperson::Model> for User {
    fn from(user: rvperson::Model) -> Self {
        Self {
            id: user.userid,
            username: user.name,
            full_name: user.realname.unwrap_or("No name".to_string()),
            email: user.univident,
            role: user.roleid.into(),
            saldo: user.saldo,
            privacy_level: user.privacy_level.into(),
            password_hash: user.pass,
            rfid_hash: user.rfid,
            temp_password_hash: None,
        }
    }
}

pub struct UpdateUser {
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<Role>,
    pub saldo: Option<i32>,
    pub privacy_level: Option<PrivacyLevel>,
    pub password: Option<String>,
    pub rfid: Option<String>,
}

pub async fn find_user_by_id(user_id: i32, db: &DatabaseConnection) -> Result<User> {
    let cutoff = Utc::now() - Duration::minutes(15);

    let result = Rvperson::find_by_id(user_id)
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

    Ok(result.into())
}

pub async fn find_user_by_username(username: &str, db: &DatabaseConnection) -> Result<User> {
    let cutoff = Utc::now() - Duration::minutes(15);

    let result = Rvperson::find()
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

    Ok(result.into())
}

pub async fn find_user_by_email(email: &str, db: &DatabaseConnection) -> Result<User> {
    let cutoff = Utc::now() - Duration::minutes(15);

    let result = Rvperson::find()
        .find_also_related(TempPassword)
        .filter(
            Condition::all()
                .add(rvperson::Column::Univident.eq(email))
                .add(
                    Condition::any()
                        .add(temppassword::Column::CreatedAt.gte(cutoff))
                        .add(temppassword::Column::Userid.is_null()),
                ),
        )
        .one(db)
        .await?
        .ok_or(AppError::Internal(anyhow::format_err!(
            "Cannot find user by email {} from database",
            email
        )))?;

    Ok(result.into())
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

async fn migrate_rfid_hash(
    rfid: &str,
    config: &AppConfig,
    db: &DatabaseConnection,
) -> Result<Option<User>> {
    let rfid_hash = old_rfid_hash(rfid)?;

    let result = Rvperson::find()
        .filter(rvperson::Column::Rfid.eq(rfid_hash))
        .one(db)
        .await?;

    match result {
        Some(result) => {
            let user = update_user(
                result.userid,
                UpdateUser {
                    username: None,
                    full_name: None,
                    email: None,
                    role: None,
                    saldo: None,
                    privacy_level: None,
                    password: None,
                    rfid: Some(rfid.to_string()),
                },
                config,
                db,
            )
            .await?;

            tracing::info!("Migrated user: {} rfid has to use bcrypt", user.username);
            Ok(Some(user))
        }
        None => Ok(None),
    }
}

pub async fn find_by_rfid(
    rfid: &str,
    config: &AppConfig,
    db: &DatabaseConnection,
) -> Result<Option<User>> {
    let rfid_hash = new_rfid_hash(rfid, config)?;

    let user = Rvperson::find()
        .find_also_related(TempPassword)
        .filter(rvperson::Column::Rfid.eq(rfid_hash))
        .one(db)
        .await?;

    match user {
        Some(user) => return Ok(Some(user.into())),
        None => migrate_rfid_hash(rfid, config, db).await,
    }
}

pub async fn remove_temp_password(user_id: i32, db: &DatabaseConnection) -> Result<()> {
    let _ = TempPassword::delete_by_id(user_id).exec(db).await?;
    Ok(())
}

pub async fn update_user(
    user_id: i32,
    data: UpdateUser,
    config: &AppConfig,
    db: &DatabaseConnection,
) -> Result<User> {
    let user = Rvperson::find_by_id(user_id).one(db).await?;

    let mut user: rvperson::ActiveModel = match user {
        Some(u) => u.into(),
        None => {
            return Err(AppError::Internal(anyhow::format_err!(
                "Cannot find user by user_id {}",
                user_id
            )));
        }
    };

    if let Some(username) = data.username {
        user.name = Set(username)
    }

    if let Some(full_name) = data.full_name {
        user.realname = Set(Some(full_name))
    }

    if let Some(email) = data.email {
        user.univident = Set(email)
    }

    if let Some(role) = data.role {
        user.roleid = Set(role.into())
    }

    if let Some(saldo) = data.saldo {
        user.saldo = Set(saldo)
    }

    if let Some(privacy_level) = data.privacy_level {
        user.privacy_level = Set(privacy_level.into())
    }

    if let Some(password) = data.password {
        let hash = bcrypt::hash(password, 11)?;
        user.pass = Set(hash)
    }

    if let Some(rfid) = data.rfid {
        let hash = new_rfid_hash(&rfid, config)?;
        user.rfid = Set(Some(hash));
    }

    let user = user.update(db).await?;

    Ok(user.into())
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

async fn migrate_rfid_hash(rfid: &str, db: &DatabaseConnection) -> Result<Option<User>> {
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
            Ok(Some(User {
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
            }))
        }
        None => Ok(None),
    }
}

pub async fn find_by_rfid(
    rfid: &str,
    config: &AppConfig,
    db: &DatabaseConnection,
) -> Result<Option<User>> {
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
        Some((user, temp_password)) => Ok(Some(User {
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
        })),
        None => migrate_rfid_hash(rfid, db).await,
    }
}

pub async fn remove_temp_password(user_id: i32, db: &DatabaseConnection) -> Result<()> {
    let _ = TempPassword::delete_by_id(user_id).exec(db).await?;
    Ok(())
}
