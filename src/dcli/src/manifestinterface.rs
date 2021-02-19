/*
* Copyright 2021 Mike Chambers
* https://github.com/mikechambers/dcli
*
* Permission is hereby granted, free of charge, to any person obtaining a copy of
* this software and associated documentation files (the "Software"), to deal in
* the Software without restriction, including without limitation the rights to
* use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
* of the Software, and to permit persons to whom the Software is furnished to do
* so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
* FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
* COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
* IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
* CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use std::path::PathBuf;
use std::str::FromStr;

use futures::TryStreamExt;
use serde_derive::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::Row;
use sqlx::{ConnectOptions, Connection, SqliteConnection};
use std::collections::HashMap;

use crate::error::Error;
use crate::manifest::definitions::{
    ActivityDefinitionData, ActivityTypeDefinitionData,
    DestinationDefinitionData, DisplayPropertiesData,
    HistoricalStatsDefinition, InventoryItemDefinitionData,
    PlaceDefinitionData,
};

pub const MANIFEST_FILE_NAME: &str = "manifest.sqlite3";

/// Takes a Destiny 2 API has and converts it to a Destiny 2 manifest db index value
pub fn convert_hash_to_id(hash: u32) -> i64 {
    let mut id: i64 = hash as i64;

    if (id & (1 << (32 - 1))) != 0 {
        id -= 1 << 32;
    }

    id
}

pub struct ManifestInterface {
    manifest_db: SqliteConnection,
    activity_definition_cache: HashMap<i64, ActivityDefinitionData>,
    inventory_item_definition_cache: HashMap<i64, InventoryItemDefinitionData>,
    historical_stats_definition_cache:
        HashMap<String, HistoricalStatsDefinition>,
}

impl ManifestInterface {
    pub async fn new(
        manifest_dir: &PathBuf,
        cache: bool,
    ) -> Result<ManifestInterface, Error> {
        let manifest_path = manifest_dir.join(MANIFEST_FILE_NAME);

        if !manifest_path.exists() {
            return Err(Error::IoFileDoesNotExist {
                description: format!(
                    "Manifest path points to non-existent file. {}",
                    manifest_path.display()
                ),
            });
        }

        let path: String = format!("{}", manifest_path.display());
        let mut read_only = true;
        let connection_string: &str = if cache {
            read_only = false;
            "sqlite:file::memory:"
        } else {
            &path
        };

        //note, we cant use WAL journal mode, which is default
        //as it can causes errors when opening a DB in readonly mode
        //We use Memory which should provide better performance
        //since we never write to the DB
        let db = SqliteConnectOptions::from_str(&connection_string)?
            .journal_mode(SqliteJournalMode::Memory)
            .read_only(read_only)
            .connect()
            .await?;

        /*
        if cache {
            match sqlx::query("ATTACH DATABASE '?' as 'tmpDb'")
                .bind(path)
                .execute(&mut db)
                .await
            {
                Ok(e) => e,
                Err(e) => {
                    db.close().await?;
                    return Err(Error::from(e));
                }
            };


            //TODO: Need to impliment this to dynamically pull table names
            //"SELECT name FROM sqlite_master WHERE type='table'"
            let table_name: String = "InventoryItemDefinition".to_string();
            //todo: do we need to pass table_name twice?
            match sqlx::query("CREATE TABLE ? AS SELECT * FROM tmpDb.?")
                .bind(table_name)
                .execute(&mut db)
                .await
            {
                Ok(e) => e,
                Err(e) => {
                    db.close().await?;
                    return Err(Error::from(e));
                }
            };

            match sqlx::query("DETACH DATABASE tmpDb").execute(&mut db).await {
                Ok(e) => e,
                Err(e) => {
                    db.close().await?;
                    return Err(Error::from(e));
                }
            }
        }
        */

        Ok(ManifestInterface {
            manifest_db: db,
            activity_definition_cache: HashMap::new(),
            inventory_item_definition_cache: HashMap::new(),
            historical_stats_definition_cache: HashMap::new(),
        })
    }

    ///closes the database connection and takes ownership of self
    pub async fn close(self) -> Result<(), Error> {
        //can call ping to see if its still open? but that throws an error if it
        //isnt, so we can just try and close
        //TODO: should we bubble the error? or just silently fail?
        self.manifest_db.close().await?;
        Ok(())
    }

    /// Searches entire manifest for id, and returns associated data for it.
    /// returns an error if more that one result found.
    //TODO: should we return a vector in case there are multiple results?
    pub async fn find(&mut self, hash: u32) -> Result<Vec<FindResult>, Error> {
        let id = convert_hash_to_id(hash);

        let tables: Vec<String> = self.get_tables_with_id_column().await?;
        let mut out: Vec<FindResult> = Vec::new();

        for table in tables.iter() {
            //for some reason sqlx doesnt let you bind table names
            let q = format!("SELECT json FROM {} WHERE id=?", table);

            let mut rows =
                sqlx::query(&q).bind(id).fetch(&mut self.manifest_db);

            while let Some(row) = rows.try_next().await? {
                // map the row into a user-defined domain type
                let json: &str = row.try_get("json")?;

                let mut v: FindResult = serde_json::from_str(json)?;
                v.raw_json = json.to_string();
                out.push(v);
            }
        }

        Ok(out)
    }

    pub async fn get_tables_with_id_column(
        &mut self,
    ) -> Result<Vec<String>, Error> {
        let mut tables: Vec<String> = Vec::new();

        //select all of the tables which have an id column
        let mut rows = sqlx::query("SELECT m.name as name, p.name as id FROM sqlite_master AS m JOIN pragma_table_info(m.name) AS p WHERE p.name = 'id'")
            .fetch(&mut self.manifest_db);

        while let Some(row) = rows.try_next().await? {
            let name: &str = row.try_get("name")?;
            tables.push(name.to_string());
        }
        Ok(tables)
    }

    pub async fn get_tables(&mut self) -> Result<Vec<String>, Error> {
        let mut tables: Vec<String> = Vec::new();

        let mut rows =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
                .fetch(&mut self.manifest_db);

        while let Some(row) = rows.try_next().await? {
            let name: &str = row.try_get("name")?;

            tables.push(name.to_string());
        }
        Ok(tables)
    }

    pub async fn get_activity_definition(
        &mut self,
        id: u32,
    ) -> Result<Option<ActivityDefinitionData>, Error> {
        let id = convert_hash_to_id(id);

        if self.activity_definition_cache.contains_key(&id) {
            let out = self.activity_definition_cache.get(&id).unwrap();

            return Ok(Some(out.clone()));
        }

        let query = &format!(
            "SELECT json FROM DestinyActivityDefinition WHERE id = {}",
            id
        );
        let data: Option<ActivityDefinitionData> =
            self.get_definition(query).await?;

        if data.is_some() {
            self.activity_definition_cache
                .insert(id, data.as_ref().unwrap().clone());
        }

        Ok(data)
    }

    //might be able to make this generic
    pub async fn get_iventory_item_definition(
        &mut self,
        id: u32,
    ) -> Result<Option<InventoryItemDefinitionData>, Error> {
        let id = convert_hash_to_id(id);

        if self.inventory_item_definition_cache.contains_key(&id) {
            let out = self.inventory_item_definition_cache.get(&id).unwrap();

            return Ok(Some(out.clone()));
        }

        let query = &format!(
            "SELECT json FROM DestinyInventoryItemDefinition WHERE id = {}",
            id
        );

        let data: Option<InventoryItemDefinitionData> =
            self.get_definition(query).await?;

        if data.is_some() {
            self.inventory_item_definition_cache
                .insert(id, data.as_ref().unwrap().clone());
        }

        Ok(data)
    }

    pub async fn get_historical_stats_definition(
        &mut self,
        id: &str,
    ) -> Result<Option<HistoricalStatsDefinition>, Error> {
        //let key = &(*id).clone().to_string();
        let key = id;
        if self.historical_stats_definition_cache.contains_key(key) {
            let out = self.historical_stats_definition_cache.get(key).unwrap();
            return Ok(Some(out.clone()));
        }

        let query = &format!(
            "SELECT json FROM DestinyHistoricalStatsDefinition WHERE key = '{}'",
            key
        );

        let data: Option<HistoricalStatsDefinition> =
            self.get_definition(query).await?;

        if data.is_some() {
            self.historical_stats_definition_cache
                .insert(key.to_string(), data.as_ref().unwrap().clone());
        }

        Ok(data)
    }

    pub async fn get_destination_definition(
        &mut self,
        id: u32,
    ) -> Result<Option<DestinationDefinitionData>, Error> {
        let id = convert_hash_to_id(id);

        let query = &format!(
            "SELECT json FROM DestinyDestinationDefinition WHERE id = {}",
            id
        );
        let data: Option<DestinationDefinitionData> =
            self.get_definition(query).await?;

        Ok(data)
    }

    pub async fn get_place_definition(
        &mut self,
        id: u32,
    ) -> Result<Option<PlaceDefinitionData>, Error> {
        let id = convert_hash_to_id(id);

        let query = &format!(
            "SELECT json FROM DestinyPlaceDefinition WHERE id = {}",
            id
        );
        let data: Option<PlaceDefinitionData> =
            self.get_definition(query).await?;

        Ok(data)
    }

    pub async fn get_activity_type_definition(
        &mut self,
        id: u32,
    ) -> Result<Option<ActivityTypeDefinitionData>, Error> {
        let id = convert_hash_to_id(id);

        let query = &format!(
            "SELECT json FROM DestinyActivityTypeDefinition WHERE id = {}",
            id
        );
        let data: Option<ActivityTypeDefinitionData> =
            self.get_definition(query).await?;

        Ok(data)
    }

    async fn get_definition<T: serde::de::DeserializeOwned>(
        &mut self,
        query: &str,
    ) -> Result<Option<T>, Error> {
        let rows = sqlx::query(query).fetch_all(&mut self.manifest_db).await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let row = &rows[0];
        let json: &str = row.try_get_unchecked("json")?;

        let data: T = serde_json::from_str(json)?;

        Ok(Some(data))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FindResult {
    #[serde(skip)]
    pub raw_json: String,

    #[serde(rename = "displayProperties")]
    pub display_properties: DisplayPropertiesData,
}
