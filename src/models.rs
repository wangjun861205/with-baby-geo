use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Location<I> {
    pub id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub geo_index: I,
}

#[derive(Serialize, Deserialize)]
pub struct LocationQuery<I> {
    geo_index: I,
    get_indices: Vec<I>,
}

#[derive(Serialize, Deserialize)]
pub struct LocationCommand<I> {
    pub latitude: f64,
    pub longitude: f64,
    pub geo_index: I,
    pub uid: String,
}
