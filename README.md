# Sea Orm Cli Bug

## Description

TL;DR: Sea Orm Cli considers the Postgres Money type to be compatible with Decimal and generates a model that cannot be selected because Sqlx considers these types incompatible.

For my project, I am using Postgres. And for the experience, I decided to use the column type [Money](https://www.postgresql.org/docs/current/datatype-money.html). But it turned out that the number of decimal places for this type is determined by the locale of the machine on which Postgres is running.

When working with this type through Sqlx, it is necessary to know the exact number of decimal places to convert PgMoney to the familiar Decimal or BigDecimal. See functions [PgMoney::to_decimal](https://github.com/launchbadge/sqlx/blob/1678b19a4672fd6a18b4891c53bf0b57638b92a4/sqlx-postgres/src/types/money.rs#L73) and [PgMoney::to_bigdecimal](https://github.com/launchbadge/sqlx/blob/1678b19a4672fd6a18b4891c53bf0b57638b92a4/sqlx-postgres/src/types/money.rs#L73). Perhaps this is the reason why Decimal is not declared as compatible with the Money type, see [description of the Postgres type for Decimal](https://github.com/launchbadge/sqlx/blob/1678b19a4672fd6a18b4891c53bf0b57638b92a4/sqlx-postgres/src/types/rust_decimal.rs#L12).

But working with Sqlx is not very convenient, so I decided to use Sea Orm. To avoid writing models by hand, I used Sea Orm Cli. To my surprise, a field of type Decimal was generated for the column with the Money type. And at runtime, when trying to select values, an "mismatched types" error occurred:

```
mismatched types; Rust type `core::option::Option<rust_decimal::decimal::Decimal>` (as SQL type `NUMERIC`) is not compatible with SQL type `MONEY`
```

In my opinion, this is a bug. Sea Orm Cli should take into account which types are compatible with each other from the perspective of Sqlx. And Sea Orm itself should 'interfere' when trying to use incompatible types.

I have no ideas on how this could be implemented. But I think it's important to document this behavior. According to my search, I am not the first one to encounter this problem. It has already been discussed here: https://github.com/SeaQL/sea-orm/discussions/1997.

## Steps to Reproduce

### Requirements

- postgres 10+
- sea-orm-cli 1.1.0

### Steps

0. Create a new database in Postgres for testing purposes.

   For example, `sea_orm_bug`. I did this in DBeaver Community.

1. Clone [repository](https://github.com/vklachkov/sea-orm-money-bug.git) and navigate to the directory.

   `git clone https://github.com/vklachkov/sea-orm-money-bug.git && cd sea-orm-money-bug`

2. Set the environment variable with the connection schema for the database.

   For example: `export DATABASE_URL="postgres://postgres:postres@127.0.0.1:5432/sea_orm_bug"`

3. Run the script `scripts/apply_migrations.sh` to apply migrations or run command `cargo run --package backend-migrator -- up`.

   After that, the migration utility will be built, migrations will be applied, and you should see:

   ```
   Applying all pending migrations
   Applying migration 'm20241214_183001_create_table'
   Migration 'm20241214_183001_create_table' has been applied
   ```

4. Run the script `scripts/generate_entities.sh` to generate entities or run command `sea-orm-cli generate entity --lib --output-dir ./entity/src`.

   You can check that everything was successful by looking at the file `entity/src/transaction.rs`. It should contain the following:

   ```rust
   #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
   #[sea_orm(table_name = "transaction")]
   pub struct Model {
     #[sea_orm(primary_key)]
     pub id: i32,
     pub amount: Decimal,
   }

   // Some trait impls
   ```

5. Now you can run the application and see the panic. To do this, run the script `scripts/run_broken_code.sh` or the command `cargo run --package backend`.

   ```rust
   thread 'main' panicked at app/src/main.rs:20:50:
   called `Result::unwrap()` on an `Err` value: Query(SqlxError(ColumnDecode { index: "\"amount\"", source: "mismatched types; Rust type `core::option::Option<rust_decimal::decimal::Decimal>` (as SQL type `NUMERIC`) is not compatible with SQL type `MONEY`" }))
   note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
   ```

### Expected Behavior

1. sea-orm-cli will not generate code that returns an error at runtime.
2. sea-orm-codegen will respect the type compatibility described in sqlx during code generation.

### Actual Behavior

sea-orm-cli (sea-orm-codegen) generates a model with an incompatible Decimal type for a column with the Money type.

### Workarounds

Don't use Postgres' money type.
