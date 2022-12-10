use sqlite::Row;
pub trait Model {
    fn from_row(row: Row) -> Self;
}
