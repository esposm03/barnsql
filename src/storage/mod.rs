mod table;
mod typ;
mod val;
pub use table::Table;
pub use typ::Typ;
pub use val::Val;

pub type ColumnName = sled::IVec;
pub type ColumnList = Vec<(ColumnName, Typ)>;
