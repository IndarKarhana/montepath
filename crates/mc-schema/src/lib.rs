pub mod builder;
pub mod compat;
pub mod diagnostics;
pub mod schema;
pub mod validate;

pub use builder::SimulationSpecBuilder;
pub use compat::{check_schema_compatibility, CompatibilityReport};
pub use diagnostics::{Diagnostic, Severity};
pub use schema::*;
pub use validate::validate_simulation_spec;
