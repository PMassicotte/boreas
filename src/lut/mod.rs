pub mod sunpos;
pub mod lookup_table;

// Re-export the main structures for convenience
pub use lookup_table::Lut;
pub use sunpos::{sunpos, sunpos_simple, SolarPosition};