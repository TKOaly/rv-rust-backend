use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Role::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Role::Roleid)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Role::Role).string_len(32).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Rvperson::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Rvperson::Userid)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Rvperson::Createdate).date_time().not_null())
                    .col(ColumnDef::new(Rvperson::Roleid).integer().not_null())
                    .col(ColumnDef::new(Rvperson::Name).string_len(64).not_null())
                    .col(
                        ColumnDef::new(Rvperson::Univident)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Rvperson::Pass).string_len(100).not_null())
                    .col(ColumnDef::new(Rvperson::Saldo).integer().not_null())
                    .col(ColumnDef::new(Rvperson::Realname).string_len(128).null())
                    .col(ColumnDef::new(Rvperson::Rfid).text().null())
                    .col(
                        ColumnDef::new(Rvperson::PrivacyLevel)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Rvperson::Table, Rvperson::Roleid)
                            .to(Role::Table, Role::Roleid),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rvperson_name")
                    .table(Rvperson::Table)
                    .col(Rvperson::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_rvperson_name")
                    .table(Rvperson::Table)
                    .col(Rvperson::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_rvperson_univident")
                    .table(Rvperson::Table)
                    .col(Rvperson::Univident)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_rvperson_rfid")
                    .table(Rvperson::Table)
                    .col(Rvperson::Rfid)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Rvperson::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Role::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Role {
    Table,
    Roleid,
    Role,
}

#[derive(DeriveIden)]
pub enum Rvperson {
    Table,
    Userid,
    Createdate,
    Roleid,
    Name,
    Univident,
    Pass,
    Saldo,
    Realname,
    Rfid,
    PrivacyLevel,
}
