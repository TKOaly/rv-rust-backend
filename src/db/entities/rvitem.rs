use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "RVITEM")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub itemid: i32,
    pub pgrpid: i32,
    pub descr: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::boxhistory::Entity")]
    Boxhistory,
    #[sea_orm(has_many = "super::itemhistory::Entity")]
    Itemhistory,
    #[sea_orm(has_many = "super::price::Entity")]
    Price,
    #[sea_orm(
        belongs_to = "super::prodgroup::Entity",
        from = "Column::Pgrpid",
        to = "super::prodgroup::Column::Pgrpid",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Prodgroup,
}

impl Related<super::boxhistory::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Boxhistory.def()
    }
}

impl Related<super::itemhistory::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Itemhistory.def()
    }
}

impl Related<super::price::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Price.def()
    }
}

impl Related<super::prodgroup::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Prodgroup.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
