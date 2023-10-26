// use std::fmt::Debug;
// use std::sync::Arc;

// use axum::async_trait;
// use sqlx::FromRow;

// // todo: should be a UUID but Sqlite needs a from UUID implementation
// #[derive(FromRow)]
// pub struct BareId {
//     pub id: String,
// }

// pub type Database = Arc<dyn Db + Send + Sync>;

// pub async fn connect(db_url: &str) -> DbResult<Database> {
//     // todo: I should figure out a way to delay the actual connection and running of migrations,
//     // and reflect the service being unavailable in the readiness check until they're complete. If
//     // our connection fails we should try a couple of times with a backoff before failing the
//     // entire service...
//     //
//     // maybe a tokio task with a channel or shared state directly that can be consumed by the
//     // healthcheck and database extractor... Maybe this state belongs on the database executor
//     // itself...

//     #[cfg(feature = "postgres")]
//     if db_url.starts_with("postgres://") {
//         let db = postgres::PostgresDb::connect(db_url).await?;
//         db.migrate().await?;
//         return Ok(Arc::new(db));
//     }

//     #[cfg(feature = "sqlite")]
//     if db_url.starts_with("sqlite://") {
//         let db = sqlite::SqliteDb::connect(db_url).await?;
//         db.migrate().await?;
//         return Ok(Arc::new(db));
//     }

//     panic!("unknown database type, unable to setup database");
// }

// #[async_trait]
// pub trait Db: Debug {
//     fn ex(&self) -> Executor;

//     async fn begin(&self) -> DbResult<TxExecutor>;
// }

// #[derive(Debug, thiserror::Error)]
// pub enum sqlx::Error {
//     #[error("query to database contained invalid syntax")]
//     BadSyntax(sqlx::Error),

//     #[error("unable to load data from database, appears to be invalid")]
//     CorruptData(sqlx::Error),

//     #[error("unable to communicate with the database")]
//     DatabaseUnavailable(sqlx::Error),

//     #[error("an internal database error occurred")]
//     InternalError(sqlx::Error),

//     #[error("error occurred while attempting database migration")]
//     MigrationFailed(sqlx::migrate::MigrateError),

//     #[error("unable to create record as it would violate a uniqueness constraint")]
//     RecordExists,

//     #[error("unable to locate record or associated foreign key")]
//     RecordNotFound,
// }

// pub type DbResult<T = ()> = Result<T, sqlx::Error>;

// pub enum Executor {
//     #[cfg(feature = "postgres")]
//     Postgres(postgres::PostgresExecutor),

//     #[cfg(feature = "sqlite")]
//     Sqlite(sqlite::SqliteExecutor),
// }

// pub struct TxExecutor(Executor);

// impl TxExecutor {
//     pub fn ex(&mut self) -> &mut Executor {
//         &mut self.0
//     }

//     pub async fn commit(self) -> DbResult {
//         match self.0 {
//             #[cfg(feature = "postgres")]
//             Executor::Postgres(e) => e.commit().await,

//             #[cfg(feature = "sqlite")]
//             Executor::Sqlite(e) => e.commit().await,
//         }
//     }
// }

// #[cfg(feature = "postgres")]
// pub mod postgres {
//     use std::str::FromStr;

//     use axum::async_trait;
//     use futures::future::BoxFuture;
//     use sqlx::migrate::Migrator;
//     use sqlx::postgres::{PgConnectOptions, PgDatabaseError, PgPool, Postgres};
//     use sqlx::Transaction;

//     use super::{Db, sqlx::Error, DbResult, Executor, TxExecutor};

//     static MIGRATOR: Migrator = sqlx::migrate!("migrations/postgres");

//     #[derive(Debug)]
//     pub enum PostgresExecutor {
//         PoolExec(PgPool),
//         TxExec(Transaction<'static, Postgres>),
//     }

//     impl PostgresExecutor {
//         pub async fn commit(self) -> DbResult {
//             match self {
//                 Self::PoolExec(_) => unreachable!("need to check this, but it shouldn't be called"),
//                 Self::TxExec(tx) => tx.commit().await.map_err(map_sqlx_error),
//             }
//         }
//     }

//     impl<'c> sqlx::Executor<'c> for &'c mut PostgresExecutor {
//         type Database = Postgres;

//         fn describe<'e, 'q: 'e>(
//             self,
//             sql: &'q str,
//         ) -> BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
//         where
//             'c: 'e,
//         {
//             match self {
//                 PostgresExecutor::PoolExec(pool) => pool.describe(sql),
//                 PostgresExecutor::TxExec(ref mut tx) => tx.describe(sql),
//             }
//         }

//         fn fetch_many<'e, 'q: 'e, E: 'q>(
//             self,
//             query: E,
//         ) -> futures::stream::BoxStream<
//             'e,
//             Result<
//                 sqlx::Either<
//                     <Self::Database as sqlx::Database>::QueryResult,
//                     <Self::Database as sqlx::Database>::Row,
//                 >,
//                 sqlx::Error,
//             >,
//         >
//         where
//             'c: 'e,
//             E: sqlx::Execute<'q, Self::Database>,
//         {
//             match self {
//                 PostgresExecutor::PoolExec(pool) => pool.fetch_many(query),
//                 PostgresExecutor::TxExec(ref mut tx) => tx.fetch_many(query),
//             }
//         }

//         fn fetch_optional<'e, 'q: 'e, E: 'q>(
//             self,
//             query: E,
//         ) -> BoxFuture<'e, Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>>
//         where
//             'c: 'e,
//             E: sqlx::Execute<'q, Self::Database>,
//         {
//             match self {
//                 PostgresExecutor::PoolExec(pool) => pool.fetch_optional(query),
//                 PostgresExecutor::TxExec(ref mut tx) => tx.fetch_optional(query),
//             }
//         }

//         fn prepare_with<'e, 'q: 'e>(
//             self,
//             sql: &'q str,
//             parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
//         ) -> BoxFuture<
//             'e,
//             Result<<Self::Database as sqlx::database::HasStatement<'q>>::Statement, sqlx::Error>,
//         >
//         where
//             'c: 'e,
//         {
//             match self {
//                 PostgresExecutor::PoolExec(pool) => pool.prepare_with(sql, parameters),
//                 PostgresExecutor::TxExec(ref mut tx) => tx.prepare_with(sql, parameters),
//             }
//         }
//     }

//     #[derive(Clone, Debug)]
//     pub struct PostgresDb {
//         pool: PgPool,
//     }

//     impl PostgresDb {
//         pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
//             let connection_options = PgConnectOptions::from_str(url)
//                 .map_err(sqlx::Error::DatabaseUnavailable)?
//                 .application_name(env!("CARGO_PKG_NAME"))
//                 .statement_cache_capacity(250);

//             let pool = sqlx::postgres::PgPoolOptions::new()
//                 .idle_timeout(std::time::Duration::from_secs(90))
//                 .max_lifetime(std::time::Duration::from_secs(1_800))
//                 .min_connections(1)
//                 .max_connections(16)
//                 .connect_with(connection_options)
//                 .await
//                 .map_err(sqlx::Error::DatabaseUnavailable)?;

//             Ok(Self { pool })
//         }

//         pub async fn migrate(&self) -> DbResult {
//             MIGRATOR
//                 .run(&self.pool)
//                 .await
//                 .map_err(sqlx::Error::MigrationFailed)
//         }
//     }

//     impl PostgresDb {
//         pub fn typed_ex(&self) -> PostgresExecutor {
//             PostgresExecutor::PoolExec(self.pool.clone())
//         }
//     }

//     #[async_trait]
//     impl Db for PostgresDb {
//         fn ex(&self) -> Executor {
//             Executor::Postgres(self.typed_ex())
//         }

//         async fn begin(&self) -> DbResult<TxExecutor> {
//             let tx = self.pool.begin().await.map_err(map_sqlx_error)?;
//             Ok(TxExecutor(Executor::Postgres(PostgresExecutor::TxExec(tx))))
//         }
//     }

//     pub fn map_sqlx_error(err: sqlx::Error) -> sqlx::Error {
//         match err {
//             sqlx::Error::ColumnDecode { .. } => sqlx::Error::CorruptData(err),
//             sqlx::Error::Database(ref db_err) => {
//                 match db_err.downcast_ref::<PgDatabaseError>().code() {
//                     "23503" /* foreign key violation */ => sqlx::Error::RecordNotFound,
//                     "23505" /* unique violation */ => sqlx::Error::RecordExists,
//                     "42601" /* syntax error */ => sqlx::Error::BadSyntax(err),
//                     "53300" /* too many connections */ => sqlx::Error::DatabaseUnavailable(err),
//                     _ => sqlx::Error::InternalError(err),
//                 }
//             }
//             sqlx::Error::PoolTimedOut => sqlx::Error::DatabaseUnavailable(err),
//             sqlx::Error::RowNotFound => sqlx::Error::RecordNotFound,
//             err => sqlx::Error::InternalError(err),
//         }
//     }
// }

// #[cfg(feature = "sqlite")]
// pub mod sqlite {
//     use std::str::FromStr;

//     use axum::async_trait;
//     use futures::future::BoxFuture;
//     use sqlx::migrate::Migrator;
//     use sqlx::sqlite::{
//         Sqlite, SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqliteSynchronous,
//     };
//     use sqlx::Transaction;

//     use super::{Db, sqlx::Error, DbResult, Executor, TxExecutor};

//     static MIGRATOR: Migrator = sqlx::migrate!("migrations/sqlite");

//     #[derive(Clone, Debug)]
//     pub struct SqliteDb {
//         pool: SqlitePool,
//     }

//     impl SqliteDb {
//         pub async fn connect(url: &str) -> DbResult<Self> {
//             let connection_options = SqliteConnectOptions::from_str(url)
//                 .map_err(sqlx::Error::DatabaseUnavailable)?
//                 .create_if_missing(true)
//                 .journal_mode(SqliteJournalMode::Wal)
//                 .statement_cache_capacity(250)
//                 .synchronous(SqliteSynchronous::Normal);

//             let pool = sqlx::sqlite::SqlitePoolOptions::new()
//                 .idle_timeout(std::time::Duration::from_secs(90))
//                 .max_lifetime(std::time::Duration::from_secs(1_800))
//                 .min_connections(1)
//                 .max_connections(16)
//                 .connect_with(connection_options)
//                 .await
//                 .map_err(sqlx::Error::DatabaseUnavailable)?;

//             Ok(Self { pool })
//         }

//         pub async fn migrate(&self) -> DbResult {
//             MIGRATOR
//                 .run(&self.pool)
//                 .await
//                 .map_err(sqlx::Error::MigrationFailed)
//         }

//         pub fn typed_ex(&self) -> SqliteExecutor {
//             SqliteExecutor::PoolExec(self.pool.clone())
//         }
//     }

//     #[async_trait]
//     impl Db for SqliteDb {
//         fn ex(&self) -> Executor {
//             Executor::Sqlite(SqliteExecutor::PoolExec(self.pool.clone()))
//         }

//         async fn begin(&self) -> DbResult<TxExecutor> {
//             let tx = self.pool.begin().await.map_err(map_sqlx_error)?;
//             Ok(TxExecutor(Executor::Sqlite(SqliteExecutor::TxExec(tx))))
//         }
//     }

//     #[derive(Debug)]
//     pub enum SqliteExecutor {
//         PoolExec(SqlitePool),
//         TxExec(Transaction<'static, Sqlite>),
//     }

//     impl SqliteExecutor {
//         pub async fn commit(self) -> DbResult {
//             match self {
//                 Self::PoolExec(_) => unreachable!("need to check this, but it shouldn't be called"),
//                 Self::TxExec(tx) => tx.commit().await.map_err(map_sqlx_error),
//             }
//         }
//     }

//     impl<'c> sqlx::Executor<'c> for &'c mut SqliteExecutor {
//         type Database = Sqlite;

//         fn describe<'e, 'q: 'e>(
//             self,
//             sql: &'q str,
//         ) -> BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
//         where
//             'c: 'e,
//         {
//             match self {
//                 SqliteExecutor::PoolExec(pool) => pool.describe(sql),
//                 SqliteExecutor::TxExec(ref mut tx) => tx.describe(sql),
//             }
//         }

//         fn fetch_many<'e, 'q: 'e, E: 'q>(
//             self,
//             query: E,
//         ) -> futures::stream::BoxStream<
//             'e,
//             Result<
//                 sqlx::Either<
//                     <Self::Database as sqlx::Database>::QueryResult,
//                     <Self::Database as sqlx::Database>::Row,
//                 >,
//                 sqlx::Error,
//             >,
//         >
//         where
//             'c: 'e,
//             E: sqlx::Execute<'q, Self::Database>,
//         {
//             match self {
//                 SqliteExecutor::PoolExec(pool) => pool.fetch_many(query),
//                 SqliteExecutor::TxExec(ref mut tx) => tx.fetch_many(query),
//             }
//         }

//         fn fetch_optional<'e, 'q: 'e, E: 'q>(
//             self,
//             query: E,
//         ) -> BoxFuture<'e, Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>>
//         where
//             'c: 'e,
//             E: sqlx::Execute<'q, Self::Database>,
//         {
//             match self {
//                 SqliteExecutor::PoolExec(pool) => pool.fetch_optional(query),
//                 SqliteExecutor::TxExec(ref mut tx) => tx.fetch_optional(query),
//             }
//         }

//         fn prepare_with<'e, 'q: 'e>(
//             self,
//             sql: &'q str,
//             parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
//         ) -> BoxFuture<
//             'e,
//             Result<<Self::Database as sqlx::database::HasStatement<'q>>::Statement, sqlx::Error>,
//         >
//         where
//             'c: 'e,
//         {
//             match self {
//                 SqliteExecutor::PoolExec(pool) => pool.prepare_with(sql, parameters),
//                 SqliteExecutor::TxExec(ref mut tx) => tx.prepare_with(sql, parameters),
//             }
//         }
//     }

//     pub fn map_sqlx_error(err: sqlx::Error) -> sqlx::Error {
//         match err {
//             sqlx::Error::ColumnDecode { .. } => sqlx::Error::CorruptData(err),
//             sqlx::Error::RowNotFound => sqlx::Error::RecordNotFound,
//             err if err.to_string().contains("FOREIGN KEY constraint failed") => {
//                 sqlx::Error::RecordNotFound
//             }
//             err if err.to_string().contains("UNIQUE constraint failed") => sqlx::Error::RecordExists,
//             err => sqlx::Error::InternalError(err),
//         }
//     }
// }
