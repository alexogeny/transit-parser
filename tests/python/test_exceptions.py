"""Unit tests for custom exceptions."""

from __future__ import annotations

import pytest


class TestExceptionHierarchy:
    """Tests for the exception class hierarchy."""

    def test_all_exceptions_inherit_from_base(self) -> None:
        """Test that all exceptions inherit from TransitParserError."""
        from transit_parser import (
            CalendarConversionError,
            ConversionError,
            FilterError,
            GtfsError,
            GtfsFileNotFoundError,
            GtfsParseError,
            GtfsValidationError,
            InvalidDateError,
            MappingError,
            TransitParserError,
            TxcError,
            TxcFileNotFoundError,
            TxcParseError,
            TxcValidationError,
        )

        # All should be subclasses of TransitParserError
        assert issubclass(GtfsError, TransitParserError)
        assert issubclass(GtfsFileNotFoundError, TransitParserError)
        assert issubclass(GtfsValidationError, TransitParserError)
        assert issubclass(GtfsParseError, TransitParserError)
        assert issubclass(TxcError, TransitParserError)
        assert issubclass(TxcFileNotFoundError, TransitParserError)
        assert issubclass(TxcValidationError, TransitParserError)
        assert issubclass(TxcParseError, TransitParserError)
        assert issubclass(ConversionError, TransitParserError)
        assert issubclass(MappingError, TransitParserError)
        assert issubclass(CalendarConversionError, TransitParserError)
        assert issubclass(FilterError, TransitParserError)
        assert issubclass(InvalidDateError, TransitParserError)

    def test_gtfs_exceptions_inherit_from_gtfs_error(self) -> None:
        """Test that GTFS exceptions inherit from GtfsError."""
        from transit_parser import (
            GtfsError,
            GtfsFileNotFoundError,
            GtfsParseError,
            GtfsValidationError,
        )

        assert issubclass(GtfsFileNotFoundError, GtfsError)
        assert issubclass(GtfsValidationError, GtfsError)
        assert issubclass(GtfsParseError, GtfsError)

    def test_txc_exceptions_inherit_from_txc_error(self) -> None:
        """Test that TXC exceptions inherit from TxcError."""
        from transit_parser import (
            TxcError,
            TxcFileNotFoundError,
            TxcParseError,
            TxcValidationError,
        )

        assert issubclass(TxcFileNotFoundError, TxcError)
        assert issubclass(TxcValidationError, TxcError)
        assert issubclass(TxcParseError, TxcError)


class TestExceptionAttributes:
    """Tests for exception attributes."""

    def test_gtfs_file_not_found_attributes(self) -> None:
        """Test GtfsFileNotFoundError attributes."""
        from transit_parser import GtfsFileNotFoundError

        exc = GtfsFileNotFoundError(
            "Feed not found",
            path="/path/to/gtfs",
            missing_files=["agency.txt", "stops.txt"],
        )

        assert str(exc) == "Feed not found"
        assert exc.path == "/path/to/gtfs"
        assert exc.missing_files == ["agency.txt", "stops.txt"]

    def test_gtfs_validation_error_attributes(self) -> None:
        """Test GtfsValidationError attributes."""
        from transit_parser import GtfsValidationError

        exc = GtfsValidationError(
            "Validation failed",
            errors=["Missing required field"],
            warnings=["Optional field missing"],
        )

        assert exc.errors == ["Missing required field"]
        assert exc.warnings == ["Optional field missing"]

    def test_gtfs_parse_error_attributes(self) -> None:
        """Test GtfsParseError attributes."""
        from transit_parser import GtfsParseError

        exc = GtfsParseError(
            "Parse error",
            file_name="stops.txt",
            line_number=42,
            column="stop_lat",
        )

        assert exc.file_name == "stops.txt"
        assert exc.line_number == 42
        assert exc.column == "stop_lat"

    def test_invalid_date_error_attributes(self) -> None:
        """Test InvalidDateError attributes."""
        from transit_parser import InvalidDateError

        exc = InvalidDateError(
            "Invalid date",
            date_string="not-a-date",
            expected_format="YYYY-MM-DD",
        )

        assert exc.date_string == "not-a-date"
        assert exc.expected_format == "YYYY-MM-DD"


class TestExceptionUsage:
    """Tests for using exceptions in practice."""

    def test_catch_all_transit_errors(self) -> None:
        """Test catching all transit errors with base class."""
        from transit_parser import GtfsFileNotFoundError, TransitParserError

        with pytest.raises(TransitParserError):
            raise GtfsFileNotFoundError("Feed not found")

    def test_catch_specific_gtfs_error(self) -> None:
        """Test catching specific GTFS errors."""
        from transit_parser import GtfsError, GtfsFileNotFoundError

        with pytest.raises(GtfsError):
            raise GtfsFileNotFoundError("Feed not found")

    def test_invalid_date_raised_by_filter(self, sample_gtfs_feed) -> None:
        """Test that InvalidDateError is raised for invalid dates."""
        from transit_parser import InvalidDateError
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)

        with pytest.raises(InvalidDateError) as exc_info:
            f.active_services_on("not-a-valid-date")

        assert exc_info.value.date_string == "not-a-valid-date"
        assert "YYYY-MM-DD" in exc_info.value.expected_format
