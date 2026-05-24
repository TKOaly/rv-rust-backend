use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        manager
            .create_table(
                Table::create()
                    .table(ProdgroupAll::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProdgroupAll::Pgrpid)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProdgroupAll::Descr)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProdgroupAll::Deleted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        db.execute_unprepared(
            r#"CREATE VIEW "PRODGROUP" AS SELECT * FROM "PRODGROUP_ALL" WHERE deleted IS FALSE"#,
        )
        .await?;

        manager
            .create_table(
                Table::create()
                    .table(RvitemAll::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RvitemAll::Itemid)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(RvitemAll::Pgrpid)
                            .integer()
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RvitemAll::Descr).string_len(64).not_null())
                    .col(
                        ColumnDef::new(RvitemAll::Deleted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RvitemAll::Table, RvitemAll::Pgrpid)
                            .to(ProdgroupAll::Table, ProdgroupAll::Pgrpid),
                    )
                    .to_owned(),
            )
            .await?;

        db.execute_unprepared(
            r#"CREATE VIEW "RVITEM" AS SELECT * FROM "RVITEM_ALL" WHERE deleted IS FALSE"#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(r#"DROP VIEW IF EXISTS "RVITEM""#)
            .await?;
        db.execute_unprepared(r#"DROP VIEW IF EXISTS "PRODGROUP""#)
            .await?;

        manager
            .drop_table(Table::drop().table(RvitemAll::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(ProdgroupAll::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ProdgroupAll {
    #[iden = "PRODGROUP_ALL"]
    Table,
    Pgrpid,
    Descr,
    Deleted,
}

#[derive(DeriveIden)]
pub enum RvitemAll {
    #[iden = "RVITEM_ALL"]
    Table,
    Itemid,
    Pgrpid,
    Descr,
    Deleted,
}
