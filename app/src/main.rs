use backend_entity::{prelude::*, *};
use sea_orm::{ActiveValue, Database, EntityTrait};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Ok(db_url) = std::env::var("DATABASE_URL") else {
        anyhow::bail!("Env var 'DATABASE_URL' not specified");
    };

    let db = Database::connect(db_url).await.unwrap();

    Transaction::insert(transaction::ActiveModel {
        id: ActiveValue::not_set(),
        amount: ActiveValue::set(10.into()),
    })
    .exec(&db)
    .await
    .unwrap();

    let txs = Transaction::find().all(&db).await.unwrap();
    println!("{txs:?}");

    Ok(())
}
