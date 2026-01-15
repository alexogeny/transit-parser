//! Parser traits for transit data formats.

use crate::error::ParseError;
use std::io::{Read, Write as IoWrite};
use std::path::Path;

/// Synchronous parser for transit data.
pub trait Parser: Sized {
    /// The output type of the parser.
    type Output;

    /// Parser configuration options.
    type Options: Default;

    /// Parse from a reader.
    fn parse<R: Read>(reader: R, options: Self::Options) -> Result<Self::Output, ParseError>;

    /// Parse from a file path.
    fn parse_path(path: &Path, options: Self::Options) -> Result<Self::Output, ParseError>;

    /// Parse from bytes.
    fn parse_bytes(bytes: &[u8], options: Self::Options) -> Result<Self::Output, ParseError> {
        Self::parse(std::io::Cursor::new(bytes), options)
    }

    /// Parse from a string.
    fn parse_str(s: &str, options: Self::Options) -> Result<Self::Output, ParseError> {
        Self::parse_bytes(s.as_bytes(), options)
    }
}

/// Streaming parser for large datasets.
pub trait StreamingParser: Sized {
    /// The item type yielded by the parser.
    type Item;

    /// Parser configuration options.
    type Options: Default;

    /// Create a new streaming parser from a reader.
    fn new<R: Read + 'static>(reader: R, options: Self::Options) -> Result<Self, ParseError>;

    /// Get the next item.
    fn next_item(&mut self) -> Result<Option<Self::Item>, ParseError>;

    /// Collect all items into a vector.
    fn collect_all(&mut self) -> Result<Vec<Self::Item>, ParseError> {
        let mut items = Vec::new();
        while let Some(item) = self.next_item()? {
            items.push(item);
        }
        Ok(items)
    }
}

/// Writer for transit data formats.
pub trait Writer: Sized {
    /// The input type to write.
    type Input;

    /// Writer configuration options.
    type Options: Default;

    /// Write to a writer.
    fn write<W: IoWrite>(
        data: &Self::Input,
        writer: W,
        options: Self::Options,
    ) -> Result<(), ParseError>;

    /// Write to a file path.
    fn write_path(
        data: &Self::Input,
        path: &Path,
        options: Self::Options,
    ) -> Result<(), ParseError>;
}
