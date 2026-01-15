"""Custom exceptions for transit-parser.

This module defines a hierarchy of exceptions for handling errors
in transit data parsing and conversion operations.

Exception hierarchy:
    TransitParserError (base)
    ├── GtfsError
    │   ├── GtfsFileNotFoundError
    │   ├── GtfsValidationError
    │   └── GtfsParseError
    ├── TxcError
    │   ├── TxcFileNotFoundError
    │   ├── TxcValidationError
    │   └── TxcParseError
    ├── ConversionError
    │   ├── MappingError
    │   └── CalendarConversionError
    └── FilterError
        └── InvalidDateError

Example usage:

    from transit_parser import GtfsFeed
    from transit_parser.exceptions import GtfsFileNotFoundError, GtfsParseError

    try:
        feed = GtfsFeed.from_path("/nonexistent/path")
    except GtfsFileNotFoundError as e:
        print(f"Feed not found: {e}")
    except GtfsParseError as e:
        print(f"Parse error: {e}")
"""

from __future__ import annotations


class TransitParserError(Exception):
    """Base exception for all transit-parser errors.

    All exceptions raised by this library inherit from this class,
    allowing you to catch all transit-parser errors with a single
    except clause.
    """

    pass


# =============================================================================
# GTFS Errors
# =============================================================================


class GtfsError(TransitParserError):
    """Base exception for GTFS-related errors."""

    pass


class GtfsFileNotFoundError(GtfsError):
    """Raised when a required GTFS file cannot be found.

    Attributes:
        path: The path that was not found.
        missing_files: List of missing required files (if applicable).
    """

    def __init__(
        self,
        message: str,
        path: str | None = None,
        missing_files: list[str] | None = None,
    ) -> None:
        super().__init__(message)
        self.path = path
        self.missing_files = missing_files or []


class GtfsValidationError(GtfsError):
    """Raised when GTFS data fails validation.

    Attributes:
        errors: List of validation error messages.
        warnings: List of validation warning messages.
    """

    def __init__(
        self,
        message: str,
        errors: list[str] | None = None,
        warnings: list[str] | None = None,
    ) -> None:
        super().__init__(message)
        self.errors = errors or []
        self.warnings = warnings or []


class GtfsParseError(GtfsError):
    """Raised when GTFS data cannot be parsed.

    Attributes:
        file_name: The name of the file that failed to parse.
        line_number: The line number where the error occurred (if known).
        column: The column name where the error occurred (if known).
    """

    def __init__(
        self,
        message: str,
        file_name: str | None = None,
        line_number: int | None = None,
        column: str | None = None,
    ) -> None:
        super().__init__(message)
        self.file_name = file_name
        self.line_number = line_number
        self.column = column


# =============================================================================
# TXC Errors
# =============================================================================


class TxcError(TransitParserError):
    """Base exception for TXC (TransXChange) related errors."""

    pass


class TxcFileNotFoundError(TxcError):
    """Raised when a TXC file cannot be found.

    Attributes:
        path: The path that was not found.
    """

    def __init__(self, message: str, path: str | None = None) -> None:
        super().__init__(message)
        self.path = path


class TxcValidationError(TxcError):
    """Raised when TXC data fails validation.

    Attributes:
        schema_version: The schema version of the document.
        errors: List of validation error messages.
    """

    def __init__(
        self,
        message: str,
        schema_version: str | None = None,
        errors: list[str] | None = None,
    ) -> None:
        super().__init__(message)
        self.schema_version = schema_version
        self.errors = errors or []


class TxcParseError(TxcError):
    """Raised when TXC XML cannot be parsed.

    Attributes:
        element: The XML element where the error occurred.
        line_number: The line number in the XML file.
    """

    def __init__(
        self,
        message: str,
        element: str | None = None,
        line_number: int | None = None,
    ) -> None:
        super().__init__(message)
        self.element = element
        self.line_number = line_number


# =============================================================================
# Conversion Errors
# =============================================================================


class ConversionError(TransitParserError):
    """Base exception for conversion-related errors."""

    pass


class MappingError(ConversionError):
    """Raised when data cannot be mapped between formats.

    Attributes:
        source_type: The source data type.
        target_type: The target data type.
        field: The field that failed to map.
    """

    def __init__(
        self,
        message: str,
        source_type: str | None = None,
        target_type: str | None = None,
        field: str | None = None,
    ) -> None:
        super().__init__(message)
        self.source_type = source_type
        self.target_type = target_type
        self.field = field


class CalendarConversionError(ConversionError):
    """Raised when calendar/service data cannot be converted.

    Attributes:
        service_id: The service ID that failed to convert.
        reason: The specific reason for the failure.
    """

    def __init__(
        self,
        message: str,
        service_id: str | None = None,
        reason: str | None = None,
    ) -> None:
        super().__init__(message)
        self.service_id = service_id
        self.reason = reason


# =============================================================================
# Filter Errors
# =============================================================================


class FilterError(TransitParserError):
    """Base exception for filtering-related errors."""

    pass


class InvalidDateError(FilterError):
    """Raised when an invalid date is provided for filtering.

    Attributes:
        date_string: The invalid date string.
        expected_format: The expected date format.
    """

    def __init__(
        self,
        message: str,
        date_string: str | None = None,
        expected_format: str | None = None,
    ) -> None:
        super().__init__(message)
        self.date_string = date_string
        self.expected_format = expected_format or "YYYY-MM-DD or YYYYMMDD"
