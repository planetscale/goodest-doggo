use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Pupper {
    pub id: u64,
    pub name: String,
    pub image: String,
    pub rating: Option<f64>,
}

#[derive(Serialize)]
pub struct DataQueryResult {
    pub all_shards: Vec<Vec<String>>,
    pub individual_shards: HashMap<String, Vec<Vec<String>>>,
    pub shard_count: u8,
}

