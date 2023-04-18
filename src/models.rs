use chrono::{DateTime, Utc};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ApiDeclaration {
    pub name: String,
    pub db_name: String,
    pub tables: Vec<TableDeclaration>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct TableDeclaration {
    pub name: String,
    pub columns: Vec<ColumnDeclaration>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ColumnDeclaration {
    pub name: String,
    pub db_type: DbType,
    pub is_nullable: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum DbType {
    Uuid,
    String(StringConstraints),
    Int(RangeConstraints<usize>),
    Float(RangeConstraints<f64>),
    DateTime(RangeConstraints<DateTime<Utc>>),
    Boolean,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct StringConstraints {
    min_length: Option<usize>,
    max_length: Option<usize>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct RangeConstraints<T> {
    min_range: Option<T>,
    max_range: Option<T>,
}