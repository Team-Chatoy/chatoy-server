pub use sea_orm_migration::prelude::*;

mod m20221221_000001_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(m20221221_000001_user::Migration)]
  }
}
