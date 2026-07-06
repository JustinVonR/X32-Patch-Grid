use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct X32Console {
    pub model: String,
    pub ip: String,
    pub version: String,
    pub id: usize,
}

