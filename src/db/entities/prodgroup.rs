use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "PRODGROUP")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub pgrpid: i32,
    pub descr: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::rvitem::Entity")]
    Rvitem,
}

impl Related<super::rvitem::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rvitem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
