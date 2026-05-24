use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Price::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Price::Priceid)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Price::Barcode).string().not_null())
                    .col(ColumnDef::new(Price::Count).integer().not_null())
                    .col(ColumnDef::new(Price::Buyprice).integer().not_null())
                    .col(ColumnDef::new(Price::Sellprice).integer().not_null())
                    .col(
                        ColumnDef::new(Price::Itemid)
                            .integer()
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Price::Userid)
                            .integer()
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Price::Starttime).date_time().null())
                    .col(ColumnDef::new(Price::Endtime).date_time().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Price::Table, Price::Itemid)
                            .to(RvitemAll::Table, RvitemAll::Itemid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Price::Table, Price::Userid)
                            .to(Rvperson::Table, Rvperson::Userid),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_price_barcode")
                    .table(Price::Table)
                    .col(Price::Barcode)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_price_starttime")
                    .table(Price::Table)
                    .col(Price::Starttime)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_price_endtime")
                    .table(Price::Table)
                    .col(Price::Endtime)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Price::Table).if_exists().to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Price {
    Table,
    Priceid,
    Barcode,
    Count,
    Buyprice,
    Sellprice,
    Itemid,
    Userid,
    Starttime,
    Endtime,
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
