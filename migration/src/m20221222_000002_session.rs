use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Session::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Session::Token)
              .string_len(64)
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(Session::User).integer().not_null())
          .col(ColumnDef::new(Session::Agent).string().not_null())
          .col(ColumnDef::new(Session::Generated).timestamp().not_null())
          .col(ColumnDef::new(Session::Expired).timestamp().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Session::Table).to_owned())
      .await
  }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Session {
  Table,
  User,
  Agent,
  Token,
  Generated,
  Expired,
}
