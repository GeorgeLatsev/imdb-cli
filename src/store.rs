use flate2::bufread::GzDecoder;
use models::{HasId, HasName};
use serde::{de, Serialize};
use sled::Batch;

use crate::models;

pub fn get_last_update_hash<T: HasName>() -> Result<Option<String>, sled::Error> {
    let file_name = T::get_name();

    let db = sled::open("imdb-db")?;
    let update_tree = db.open_tree("updates")?;

    let result = update_tree
        .get(file_name.as_bytes())
        .unwrap_or(None)
        .and_then(|bytes| Some(std::str::from_utf8(&*bytes).unwrap_or("").to_owned()));

    Ok(result)
}

pub fn update<T: de::DeserializeOwned + std::fmt::Debug + Serialize + HasId + HasName>(
    bytes: bytes::Bytes,
) -> Result<(), sled::Error> {
    let file_name = T::get_name();

    let db = sled::open("imdb-db")?;
    let file_tree = db.open_tree(file_name.as_bytes())?;

    let gz = GzDecoder::new(&*bytes);

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .flexible(true)
        .from_reader(gz);

    let max_batch_size: usize = 8192;
    let mut current_batch_size: usize = 0;
    let mut batch = sled::Batch::default();

    for result in reader.deserialize() {
        let record: T = result.unwrap();
        let value = serde_json::to_string(&record).unwrap();

        batch.insert(record.get_id().as_bytes(), value.as_bytes());
        current_batch_size += 1;

        if current_batch_size == max_batch_size {
            file_tree.apply_batch(batch).unwrap();
            current_batch_size = 0;
            batch = Batch::default();
        }
    }

    if current_batch_size > 0 {
        file_tree.apply_batch(batch).unwrap();
    }

    Ok(())
}
