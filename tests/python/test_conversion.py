"""Unit tests for TXC to GTFS conversion."""

from __future__ import annotations

from pathlib import Path

import pytest


class TestTxcToGtfsConverter:
    """Tests for the TxcToGtfsConverter class."""

    def test_convert_single_document(self, sample_txc_document, temp_output_dir: Path) -> None:
        """Test converting a single TXC document to GTFS."""
        from transit_parser import TxcToGtfsConverter

        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        assert result is not None
        assert result.feed is not None

    def test_converted_feed_has_agencies(self, sample_txc_document) -> None:
        """Test that converted feed has agencies from operators."""
        from transit_parser import TxcToGtfsConverter

        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        agencies = result.feed.agencies
        assert len(agencies) >= 1

        # Check that operator was converted
        agency_names = {a.name for a in agencies}
        assert any("Sample" in name for name in agency_names)

    def test_converted_feed_has_routes(self, sample_txc_document) -> None:
        """Test that converted feed has routes from services/lines."""
        from transit_parser import TxcToGtfsConverter

        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        routes = result.feed.routes
        assert len(routes) >= 1

    def test_converted_feed_has_trips(self, sample_txc_document) -> None:
        """Test that converted feed has trips from vehicle journeys."""
        from transit_parser import TxcToGtfsConverter

        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        trips = result.feed.trips
        # 5 vehicle journeys should produce 5 trips
        assert len(trips) == 5

    def test_converted_feed_has_stops(self, sample_txc_document) -> None:
        """Test that converted feed has stops from stop points."""
        from transit_parser import TxcToGtfsConverter

        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        stops = result.feed.stops
        assert len(stops) == 4

    def test_converted_feed_has_calendar(self, sample_txc_document) -> None:
        """Test that converted feed has calendar entries."""
        from transit_parser import TxcToGtfsConverter

        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        calendars = result.feed.calendars
        assert len(calendars) >= 1

        # Should have weekday service (MondayToFriday in TXC)
        has_weekday = any(
            c.monday is True and c.friday is True and c.saturday is False
            for c in calendars
        )
        assert has_weekday

    def test_write_converted_feed_to_directory(
        self, sample_txc_document, temp_output_dir: Path
    ) -> None:
        """Test writing converted feed to a directory."""
        from transit_parser import TxcToGtfsConverter

        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        output_path = temp_output_dir / "gtfs"
        result.feed.to_path(str(output_path))

        # Check that required files exist
        assert (output_path / "agency.txt").exists()
        assert (output_path / "stops.txt").exists()
        assert (output_path / "routes.txt").exists()
        assert (output_path / "trips.txt").exists()
        assert (output_path / "stop_times.txt").exists()


class TestConversionResult:
    """Tests for ConversionResult object."""

    def test_result_has_stats(self, sample_txc_document) -> None:
        """Test that conversion result has statistics."""
        from transit_parser import TxcToGtfsConverter

        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        assert hasattr(result, "stats")

    def test_result_has_warnings(self, sample_txc_document) -> None:
        """Test that conversion result has warnings list."""
        from transit_parser import TxcToGtfsConverter

        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        assert hasattr(result, "warnings")


class TestConversionRoundTrip:
    """Tests for converting TXC to GTFS and loading result."""

    def test_converted_feed_is_loadable(
        self, sample_txc_document, temp_output_dir: Path
    ) -> None:
        """Test that converted GTFS feed can be loaded back."""
        from transit_parser import GtfsFeed, TxcToGtfsConverter

        # Convert TXC to GTFS
        converter = TxcToGtfsConverter()
        result = converter.convert(sample_txc_document)

        # Write to disk
        output_path = temp_output_dir / "gtfs"
        result.feed.to_path(str(output_path))

        # Load it back
        loaded_feed = GtfsFeed.from_path(str(output_path))

        # Verify data integrity
        assert loaded_feed.agency_count == result.feed.agency_count
        assert loaded_feed.route_count == result.feed.route_count
        assert loaded_feed.trip_count == result.feed.trip_count
