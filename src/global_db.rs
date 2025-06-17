use once_cell::sync::OnceCell;
use std::sync::Arc;
use sqlx::{Sqlite, Pool};

static DB_POOL: OnceCell<Arc<Pool<Sqlite>>> = OnceCell::new();

/// Sets the global database pool. This must be called once during initialization.
pub fn set_db_pool(pool: Arc<Pool<Sqlite>>) {
    DB_POOL.set(pool).expect("Database pool already set");
}

/// Retrieves a clone of the global database pool.
/// This function is async only because it might be used in an async context.
pub fn get_db_pool() -> Arc<Pool<Sqlite>> {
    DB_POOL.get().expect("Database pool not set").clone()
}
