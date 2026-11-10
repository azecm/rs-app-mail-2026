use deadpool_postgres::{Config, ManagerConfig, Object, Pool, PoolError, RecyclingMethod, Runtime};
use serde::Deserialize;
use tokio::sync::OnceCell;
use tokio_postgres::{NoTls, Row, types::ToSql};

const DB_PARAMS: &str = include_str!("../../env.toml");
#[derive(Debug, Deserialize)]
struct DBParams {
    db: String,
    user: String,
    password: String,
    host: String,
    port: u16,
}

static DB_POOL: OnceCell<Pool> = OnceCell::const_new();

async fn get_pool() -> Pool {
    let p = toml::from_str::<DBParams>(DB_PARAMS).unwrap();

    let mut cfg = Config::new();
    cfg.dbname = Some(p.db);
    cfg.user = Some(p.user);
    cfg.password = Some(p.password);
    cfg.host = Some(p.host);
    cfg.port = Some(p.port);

    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool: Pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    pool
}

pub async fn db_conn() -> Result<Object, PoolError> {
    (DB_POOL.get_or_init(get_pool).await).get().await
}

pub async fn db_query<R>(
    data_from: fn(Row) -> R,
    statement: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Vec<R> {
    match db_conn().await {
        Ok(db) => match db.query(statement, params).await {
            Ok(result) => result.into_iter().map(data_from).collect::<Vec<_>>(),
            Err(err) => {
                tracing::error!("db_query [statement]: {:?}", statement);
                tracing::error!("db_query [params]: {:?}", params);
                tracing::error!("db_query [error]: {:?}", err);
                Vec::new()
            }
        },
        Err(err) => {
            tracing::error!("db_conn: {:?}", err);
            Vec::new()
        }
    }
}

pub async fn db_update_query(statement: &str, params: &[&(dyn ToSql + Sync)]) -> bool {
    match db_conn().await {
        Ok(db) => match db.query(statement, params).await {
            Ok(_) => true,
            Err(err) => {
                tracing::error!("db_query [statement]: {:?}", statement);
                tracing::error!("db_query [params]: {:?}", params);
                tracing::error!("db_query [error]: {:?}", err);
                false
            }
        },
        Err(err) => {
            tracing::error!("db_conn: {:?}", err);
            false
        }
    }
}

/*
pub fn content_as_md(source: &str) -> Option<String> {
    if source.contains("</") {
        htmd::convert(source).ok()
    } else {
        Some(source.to_string())
    }
}
*/
