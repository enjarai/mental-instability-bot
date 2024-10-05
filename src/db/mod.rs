use sqlx::{Pool, Postgres};

pub(crate) type Db = Pool<Postgres>;
pub(crate) type Result<T> = core::result::Result<T, sqlx::Error>;

pub(crate) async fn init_db(db: &mut Db) -> Result<()> {
    let mut trans = db.begin().await?;
    sqlx::query!(
        "
CREATE TABLE IF NOT EXISTS Reminder (
        id int primary key,
        
)
        "
    )
    .execute(&mut *trans)
    .await?;

    Ok(())
}
