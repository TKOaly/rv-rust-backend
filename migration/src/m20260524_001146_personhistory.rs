use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Personhist::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Personhist::Pershistid)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Personhist::Time).date_time().not_null())
                    .col(ColumnDef::new(Personhist::Ipaddress).string().null())
                    .col(ColumnDef::new(Personhist::Actionid).integer().not_null())
                    .col(ColumnDef::new(Personhist::Userid1).integer().not_null())
                    .col(ColumnDef::new(Personhist::Userid2).integer().not_null())
                    .col(ColumnDef::new(Personhist::Saldhistid).integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Personhist::Table, Personhist::Actionid)
                            .to(Action::Table, Action::Actionid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Personhist::Table, Personhist::Userid1)
                            .to(Rvperson::Table, Rvperson::Userid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Personhist::Table, Personhist::Userid2)
                            .to(Rvperson::Table, Rvperson::Userid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Personhist::Table, Personhist::Saldhistid)
                            .to(Saldohistory::Table, Saldohistory::Saldhistid),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_personhist_time")
                    .table(Personhist::Table)
                    .col(Personhist::Time)
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
                        .table(Personhist::Table)
                        .if_exists()
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Personhist {
    Table,
    Pershistid,
    Time,
    Ipaddress,
    Actionid,
    Userid1,
    Userid2,
    Saldhistid,
}

#[derive(DeriveIden)]
enum Action {
    Table,
    Actionid,
}

#[derive(DeriveIden)]
enum Rvperson {
    Table,
    Userid,
}

#[derive(DeriveIden)]
enum Saldohistory {
    Table,
    Saldhistid,
}
