use std::sync::{Arc, Mutex};

use crate::database::Database;

pub async fn get_all_tracks(database: Arc<Mutex<Database>>) -> Vec<u8> {
    let tracks = database.lock().unwrap().all_tracks();
    bitcode::serialize(&tracks).unwrap()
}
