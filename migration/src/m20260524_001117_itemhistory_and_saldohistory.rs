use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Saldohistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Saldohistory::Saldhistid)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Saldohistory::Userid).integer().not_null())
                    .col(ColumnDef::new(Saldohistory::Time).date_time().not_null())
                    .col(ColumnDef::new(Saldohistory::Saldo).integer().null())
                    .col(
                        ColumnDef::new(Saldohistory::Difference)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Saldohistory::Table, Saldohistory::Userid)
                            .to(Rvperson::Table, Rvperson::Userid),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_saldohistory_time")
                    .table(Saldohistory::Table)
                    .col(Saldohistory::Time)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_saldohistory_saldo")
                    .table(Saldohistory::Table)
                    .col(Saldohistory::Saldo)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Itemhistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Itemhistory::Itemhistid)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Itemhistory::Time).date_time().not_null())
                    .col(ColumnDef::new(Itemhistory::Count).integer().null())
                    .col(
                        ColumnDef::new(Itemhistory::Actionid)
                            .integer()
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Itemhistory::Itemid)
                            .integer()
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Itemhistory::Userid)
                            .integer()
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Itemhistory::Priceid1)
                            .integer()
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Itemhistory::Priceid2)
                            .integer()
                            .unsigned()
                            .null(),
                    )
                    .col(ColumnDef::new(Itemhistory::Saldhistid).integer().null())
                    .col(
                        ColumnDef::new(Itemhistory::Itemhistid2)
                            .integer()
                            .null()
                            .unique_key(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Itemhistory::Table, Itemhistory::Actionid)
                            .to(Action::Table, Action::Actionid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Itemhistory::Table, Itemhistory::Itemid)
                            .to(RvitemAll::Table, RvitemAll::Itemid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Itemhistory::Table, Itemhistory::Userid)
                            .to(Rvperson::Table, Rvperson::Userid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Itemhistory::Table, Itemhistory::Priceid1)
                            .to(Price::Table, Price::Priceid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Itemhistory::Table, Itemhistory::Priceid2)
                            .to(Price::Table, Price::Priceid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Itemhistory::Table, Itemhistory::Saldhistid)
                            .to(Saldohistory::Table, Saldohistory::Saldhistid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Itemhistory::Table, Itemhistory::Itemhistid2)
                            .to(Itemhistory::Table, Itemhistory::Itemhistid),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_itemhistory_time")
                    .table(Itemhistory::Table)
                    .col(Itemhistory::Time)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Itemhistory::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Saldohistory::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Itemhistory {
    Table,
    Itemhistid,
    Time,
    Count,
    Actionid,
    Itemid,
    Userid,
    Priceid1,
    Priceid2,
    Saldhistid,
    Itemhistid2,
}

#[derive(DeriveIden)]
pub enum Saldohistory {
    Table,
    Saldhistid,
    Userid,
    Time,
    Saldo,
    Difference,
}

#[derive(DeriveIden)]
enum Action {
    Table,
    Actionid,
}

#[derive(DeriveIden)]
enum RvitemAll {
    #[iden = "RVITEM_ALL"]
    Table,
    Itemid,
}

#[derive(DeriveIden)]
enum Rvperson {
    Table,
    Userid,
}

#[derive(DeriveIden)]
enum Price {
    Table,
    Priceid,
}
