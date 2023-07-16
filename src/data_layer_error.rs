use std::error::Error;

/// 
/// Result type for `DataLayer`s. All `DataLayer` implementations
/// should return this type.
/// 
pub type Result<T> = std::result::Result<T, DataLayerError>;

/// 
/// Result type for `DataLayer`s. Represents a generic returned error,
/// which allows different kinds of `DataLayer` implementation.
///
pub type DataLayerError = Box<dyn Error + Send + Sync>;