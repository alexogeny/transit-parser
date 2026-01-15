//! Adapter trait for converting between transit formats.

use crate::error::AdapterError;

/// Adapter for converting between transit data formats.
pub trait Adapter<Source, Target>: Sized {
    /// Adapter configuration options.
    type Options: Default;

    /// Additional context for conversion (e.g., NaPTAN data, bank holidays).
    type Context;

    /// Convert source format to target format.
    fn convert(source: Source, options: Self::Options) -> Result<Target, AdapterError>;

    /// Convert with additional context.
    fn convert_with_context(
        source: Source,
        context: &Self::Context,
        options: Self::Options,
    ) -> Result<Target, AdapterError>;
}
