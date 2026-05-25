use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Temppassword::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Temppassword::Userid)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Temppassword::TempPassword)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Temppassword::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Temppassword::Table, Temppassword::Userid)
                            .to(Rvperson::Table, Rvperson::Userid),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if std::env::var("ENVIRONMENT").unwrap() != "production" {
            manager
                .drop_table(
                    Table::drop()
                        .table(Temppassword::Table)
                        .if_exists()
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Temppassword {
    Table,
    Userid,
    TempPassword,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Rvperson {
    Table,
    Userid,
}
