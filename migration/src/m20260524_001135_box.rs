use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Rvbox::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Rvbox::Barcode)
                            .string_len(64)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Rvbox::Itembarcode).string_len(64).not_null())
                    .col(ColumnDef::new(Rvbox::Itemcount).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rvbox_itembarcode")
                    .table(Rvbox::Table)
                    .col(Rvbox::Itembarcode)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Boxhistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Boxhistory::BoxhistoryId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Boxhistory::Time).date_time().not_null())
                    .col(ColumnDef::new(Boxhistory::Barcode).string().not_null())
                    .col(ColumnDef::new(Boxhistory::Itemid).integer().null())
                    .col(ColumnDef::new(Boxhistory::Itemcount).integer().null())
                    .col(ColumnDef::new(Boxhistory::Userid).integer().not_null())
                    .col(ColumnDef::new(Boxhistory::Actionid).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Boxhistory::Table, Boxhistory::Itemid)
                            .to(RvitemAll::Table, RvitemAll::Itemid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Boxhistory::Table, Boxhistory::Userid)
                            .to(Rvperson::Table, Rvperson::Userid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Boxhistory::Table, Boxhistory::Actionid)
                            .to(Action::Table, Action::Actionid),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_boxhistory_itemid")
                    .table(Boxhistory::Table)
                    .col(Boxhistory::Itemid)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_boxhistory_userid")
                    .table(Boxhistory::Table)
                    .col(Boxhistory::Userid)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Boxhistory::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Rvbox::Table).if_exists().to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Rvbox {
    Table,
    Barcode,
    Itembarcode,
    Itemcount,
}

#[derive(DeriveIden)]
pub enum Boxhistory {
    Table,
    BoxhistoryId,
    Time,
    Barcode,
    Itemid,
    Itemcount,
    Userid,
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
enum Action {
    Table,
    Actionid,
}
