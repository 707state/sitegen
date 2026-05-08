use idb::{Database, DatabaseEvent, Factory, ObjectStoreParams, TransactionMode};
use js_sys::Date;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use serde_wasm_bindgen::{from_value, Serializer};

const DB_NAME: &str = "sitegen_cache";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "json_cache";
const TTL_MS: f64 = 24.0 * 60.0 * 60.0 * 1000.0; // 24 hours in milliseconds

macro_rules! console_log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&JsValue::from_str(&format!("[IDB] {}", format!($($t)*))))
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    #[serde(rename = "data")]
    pub data: Vec<u8>,
    #[serde(rename = "ts")]
    pub timestamp: f64,
}

async fn open_db() -> Result<Database, String> {
    let factory = Factory::new().map_err(|e| format!("IDB factory error: {e}"))?;
    let mut db_req = factory
        .open(DB_NAME, Some(DB_VERSION))
        .map_err(|e| format!("IDB open error: {e}"))?;

    db_req.on_upgrade_needed(|event| {
        console_log!("upgrade_needed: creating object store");
        let db = event.database().unwrap();
        // Use out-of-line key: no key_path, key is supplied explicitly on put/get
        let params = ObjectStoreParams::new();
        let _ = db.create_object_store(STORE_NAME, params);
    });

    db_req.await.map_err(|e| format!("IDB await error: {e}"))
}

pub async fn get_cached<T>(url: &str) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
{
    console_log!("get_cached: checking cache for {url}");
    let db = open_db().await.map_err(|e| {
        console_log!("get_cached: open_db failed: {e}");
        e
    })?;
    let tx = db
        .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
        .map_err(|e| {
            console_log!("get_cached: transaction failed: {e}");
            format!("IDB transaction error: {e}")
        })?;
    let store = tx
        .object_store(STORE_NAME)
        .map_err(|e| {
            console_log!("get_cached: object_store failed: {e}");
            format!("IDB object_store error: {e}")
        })?;

    let get_req = store
        .get(JsValue::from_str(url))
        .map_err(|e| {
            console_log!("get_cached: get() failed: {e}");
            format!("IDB get request error: {e}")
        })?;

    match get_req.await {
        Ok(Some(value)) => {
            console_log!("get_cached: found raw value in IDB for {url}");
            match from_value::<CacheEntry>(value) {
                Err(e) => {
                    console_log!("get_cached: from_value deserialization failed: {e}");
                    Err(format!("Cache deserialize error: {e}"))
                }
                Ok(entry) => {
                    let now: f64 = Date::now();
                    let age_s = ((now - entry.timestamp) / 1000.0) as u64;
                    if now - entry.timestamp > TTL_MS {
                        console_log!("get_cached: cache expired (age={age_s}s) for {url}, deleting");
                        drop(tx);
                        let db = open_db().await?;
                        let del_tx = db
                            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
                            .map_err(|e| format!("IDB delete transaction error: {e}"))?;
                        let del_store = del_tx
                            .object_store(STORE_NAME)
                            .map_err(|e| format!("IDB delete store error: {e}"))?;
                        let del_req = del_store
                            .delete(JsValue::from_str(url))
                            .map_err(|e| format!("IDB delete request error: {e}"))?;
                        let _ = del_req.await;
                        Ok(None)
                    } else {
                        match serde_json::from_slice::<T>(&entry.data) {
                            Ok(data) => {
                                console_log!("get_cached: HIT (age={age_s}s, {} bytes) for {url}", entry.data.len());
                                Ok(Some(data))
                            }
                            Err(e) => {
                                console_log!("get_cached: serde_json::from_slice failed: {e}");
                                Err(format!("JSON parse from cache error: {e}"))
                            }
                        }
                    }
                }
            }
        }
        Ok(None) => {
            console_log!("get_cached: MISS (no entry) for {url}");
            Ok(None)
        }
        Err(e) => {
            console_log!("get_cached: get().await error: {e}");
            Err(format!("IDB get await error: {e}"))
        }
    }
}

pub async fn set_cached(url: &str, data: &[u8]) -> Result<(), String> {
    console_log!("set_cached: writing {} bytes for {url}", data.len());
    let db = open_db().await?;
    let tx = db
        .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
        .map_err(|e| format!("IDB write transaction error: {e}"))?;
    let store = tx
        .object_store(STORE_NAME)
        .map_err(|e| format!("IDB write store error: {e}"))?;

    let entry = CacheEntry {
        data: data.to_vec(),
        timestamp: Date::now(),
    };

    let js_value = entry
        .serialize(&Serializer::json_compatible())
        .map_err(|e| format!("Serialize error: {e}"))?;

    let put_req = store
        .put(&js_value, Some(&JsValue::from_str(url)))
        .map_err(|e| format!("IDB put request error: {e}"))?;
    put_req.await.map_err(|e| {
        console_log!("set_cached: put().await error: {e}");
        format!("IDB put await error: {e}")
    })?;
    console_log!("set_cached: OK for {url}");
    Ok(())
}
