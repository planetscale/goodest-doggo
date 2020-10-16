use super::CLIENT_POOL;
use super::Pool;
use mysql::prelude::FromRow;
use mysql::prelude::Queryable;
use doggo_core::collection_abstractions::DataRepository;
use doggo_core::dtos::DataQueryResult;

pub struct VitessDataRepository {
    pool: Pool,
}

impl VitessDataRepository {
    /// Associative function to create a new user repository from a connection.
    pub fn new() -> VitessDataRepository {
        VitessDataRepository {
            // "Clone" the pool (it's an Arc, so just increase count) and then get a connection for use
            // in this handler.
            pool: CLIENT_POOL.clone(),
        }
    }

    fn get_shards(&self, database: impl AsRef<str>) -> Result<Vec<String>, mysql::Error> {
        let mut conn = self.pool.get_conn().unwrap();
        let conn = conn.as_mut();
        let rows = conn.query_iter("SHOW VITESS_SHARDS")?.map(|rs| rs.into_iter()).flatten().filter_map(|row| {
            let parts = <(String,)>::from_row(row).0;
            let parts: Vec<_> = parts.split("/").collect();
            if parts.len() != 2 || parts[0] != database.as_ref() {
                return None;
            }
            Some(parts[1].to_string())
        }).collect();
        Ok(rows)
    }

    fn query_for_database(&mut self, database: impl AsRef<str>, query: impl AsRef<str>) -> Result<Vec<Vec<String>>, mysql::Error> {
        // TODO:  If a multi-part query (e.g. "SELECT 1; SELECT 2;") is
        //    included, but not all the queries have the same number of rows
        //    (e.g. "SELECT 1, 2; SELECT 3;"), the resulting front-end table
        //    will not look so great.  This could be fixed with a proper 2D
        //    matrix if the need arises.
        let mut conn = self.pool.get_conn().unwrap();
        let conn = conn.as_mut();
        conn.select_db(database.as_ref());
        let rows: Vec<_> = conn.query_iter(query)?.into_iter().map(|rs| rs.into_iter()).flatten()
            .map(|row| {
                let mut row_as_strings = Vec::with_capacity(row.len());
                for i in 0..row.len() {
                    row_as_strings.push(row[i].as_sql(true));
                }
                row_as_strings
            })
            .collect();
        Ok(rows)
    }
}

impl DataRepository for VitessDataRepository {
    type Error = mysql::Error;

    fn get(&mut self, query: impl AsRef<str>) -> Result<DataQueryResult, Self::Error> {
        Ok(DataQueryResult{
            all_shards: self.query_for_database("puppers", &query)?,
            first_shard: self.query_for_database("puppers:-80", &query)?,
            last_shard: self.query_for_database("puppers:80-", &query)?
        })
    }
}

