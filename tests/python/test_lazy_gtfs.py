"""Unit tests for lazy GTFS feed loading."""

from __future__ import annotations

from pathlib import Path

import pytest


class TestLazyGtfsFeed:
    """Tests for the LazyGtfsFeed class."""

    def test_load_from_directory(self, gtfs_fixtures_dir: Path) -> None:
        """Test loading a lazy GTFS feed from a directory."""
        from transit_parser import LazyGtfsFeed

        feed = LazyGtfsFeed.from_path(str(gtfs_fixtures_dir))
        assert feed is not None

    def test_lazy_loading_defers_parsing(self, gtfs_fixtures_dir: Path) -> None:
        """Test that lazy loading defers actual parsing until first access."""
        from transit_parser import LazyGtfsFeed

        # This should be very fast (no parsing yet)
        feed = LazyGtfsFeed.from_path(str(gtfs_fixtures_dir))

        # Feed should exist but no data parsed yet
        assert feed is not None

    def test_agencies_access(self, sample_lazy_gtfs_feed) -> None:
        """Test lazy access to agencies."""
        agencies = sample_lazy_gtfs_feed.agencies
        assert len(agencies) == 2

        # Attribute is 'id', not 'agency_id'
        agency_ids = {a.id for a in agencies}
        assert "agency_1" in agency_ids
        assert "agency_2" in agency_ids

    def test_stops_access(self, sample_lazy_gtfs_feed) -> None:
        """Test lazy access to stops."""
        stops = sample_lazy_gtfs_feed.stops
        assert len(stops) == 5

    def test_routes_access(self, sample_lazy_gtfs_feed) -> None:
        """Test lazy access to routes."""
        routes = sample_lazy_gtfs_feed.routes
        assert len(routes) == 3

    def test_trips_access(self, sample_lazy_gtfs_feed) -> None:
        """Test lazy access to trips."""
        trips = sample_lazy_gtfs_feed.trips
        assert len(trips) == 5

    def test_stop_times_access(self, sample_lazy_gtfs_feed) -> None:
        """Test lazy access to stop times."""
        stop_times = sample_lazy_gtfs_feed.stop_times
        assert len(stop_times) == 16

    def test_calendars_access(self, sample_lazy_gtfs_feed) -> None:
        """Test lazy access to calendar entries."""
        # Property is 'calendars', not 'calendar'
        calendars = sample_lazy_gtfs_feed.calendars
        assert len(calendars) == 2

    def test_calendar_dates_access(self, sample_lazy_gtfs_feed) -> None:
        """Test lazy access to calendar dates."""
        calendar_dates = sample_lazy_gtfs_feed.calendar_dates
        assert len(calendar_dates) == 3

    def test_shapes_access(self, sample_lazy_gtfs_feed) -> None:
        """Test lazy access to shapes."""
        shapes = sample_lazy_gtfs_feed.shapes
        assert len(shapes) >= 1

    def test_count_properties(self, sample_lazy_gtfs_feed) -> None:
        """Test count properties work with lazy loading."""
        # These are properties (int attributes), not methods
        assert sample_lazy_gtfs_feed.agency_count == 2
        assert sample_lazy_gtfs_feed.stop_count == 5
        assert sample_lazy_gtfs_feed.route_count == 3
        assert sample_lazy_gtfs_feed.trip_count == 5
        assert sample_lazy_gtfs_feed.stop_time_count == 16

    def test_materialize_to_regular_feed(self, sample_lazy_gtfs_feed) -> None:
        """Test converting lazy feed to regular GtfsFeed."""
        regular_feed = sample_lazy_gtfs_feed.materialize()

        # Should have same data (count properties, not methods)
        assert regular_feed.agency_count == 2
        assert regular_feed.stop_count == 5
        assert regular_feed.route_count == 3

    def test_caching_after_first_access(self, sample_lazy_gtfs_feed) -> None:
        """Test that data is cached after first access."""
        # First access triggers parsing
        agencies1 = sample_lazy_gtfs_feed.agencies

        # Second access should return cached data
        agencies2 = sample_lazy_gtfs_feed.agencies

        assert len(agencies1) == len(agencies2)
        ids1 = {a.id for a in agencies1}
        ids2 = {a.id for a in agencies2}
        assert ids1 == ids2


class TestLazyGtfsFeedEdgeCases:
    """Tests for edge cases in lazy GTFS loading."""

    def test_nonexistent_path_raises_error(self) -> None:
        """Test that loading from a nonexistent path raises an error."""
        from transit_parser import LazyGtfsFeed

        with pytest.raises(Exception):
            LazyGtfsFeed.from_path("/nonexistent/path")
