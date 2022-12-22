use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Member::Table)
          .if_not_exists()
          .col(ColumnDef::new(Member::User).integer().not_null())
          .col(ColumnDef::new(Member::Room).integer().not_null())
          .col(ColumnDef::new(Member::Joined).timestamp().not_null())
          .primary_key(Index::create().col(Member::User).col(Member::Room))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Member::Table).to_owned())
      .await
  }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Member {
  Table,
  User,
  Room,
  Joined,
}
