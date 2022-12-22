pub use sea_orm_migration::prelude::*;

mod m20221221_000001_user;
mod m20221222_000002_session;
mod m20221222_000003_room;
mod m20221222_000004_member;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20221221_000001_user::Migration),
      Box::new(m20221222_000002_session::Migration),
      Box::new(m20221222_000003_room::Migration),
      Box::new(m20221222_000004_member::Migration),
    ]
  }
}
