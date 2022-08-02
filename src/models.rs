pub struct Location<T> {
    id: T,
    latitude: f64,
    longitude: f64,
    geo_index: String,
}

pub struct LocationQuery {
    geo_index: String,
}

pub struct LocationCommand {
    latitude: f64,
    longitude: f64,
    geo_index: String,
}
