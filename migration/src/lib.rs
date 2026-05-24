pub use sea_orm_migration::prelude::*;

mod m20260524_000936_users_and_roles;
mod m20260524_001023_prodgroups_and_items;
mod m20260524_001043_price;
mod m20260524_001051_action;
mod m20260524_001117_itemhistory_and_saldohistory;
mod m20260524_001135_box;
mod m20260524_001146_personhistory;
mod m20260524_001205_preferences;
mod m20260524_001232_temp_pass;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260524_000936_users_and_roles::Migration),
            Box::new(m20260524_001023_prodgroups_and_items::Migration),
            Box::new(m20260524_001043_price::Migration),
            Box::new(m20260524_001051_action::Migration),
            Box::new(m20260524_001117_itemhistory_and_saldohistory::Migration),
            Box::new(m20260524_001135_box::Migration),
            Box::new(m20260524_001146_personhistory::Migration),
            Box::new(m20260524_001205_preferences::Migration),
            Box::new(m20260524_001232_temp_pass::Migration),
        ]
    }
}
