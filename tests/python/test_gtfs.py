"""Unit tests for GTFS feed loading and parsing."""

from __future__ import annotations

from pathlib import Path

import pytest


class TestGtfsFeed:
    """Tests for the GtfsFeed class."""

    def test_load_from_directory(self, gtfs_fixtures_dir: Path) -> None:
        """Test loading a GTFS feed from a directory."""
        from transit_parser import GtfsFeed

        feed = GtfsFeed.from_path(str(gtfs_fixtures_dir))
        assert feed is not None

    def test_agencies_loaded(self, sample_gtfs_feed) -> None:
        """Test that agencies are loaded correctly."""
        agencies = sample_gtfs_feed.agencies
        assert len(agencies) == 2

        # Check agency IDs (attribute is 'id', not 'agency_id')
        agency_ids = {a.id for a in agencies}
        assert "agency_1" in agency_ids
        assert "agency_2" in agency_ids

        # Check agency details (attributes are 'name', 'url', 'timezone')
        agency_1 = next(a for a in agencies if a.id == "agency_1")
        assert agency_1.name == "Test Transit Agency"
        assert agency_1.url == "https://example.com"
        assert agency_1.timezone == "America/New_York"

    def test_stops_loaded(self, sample_gtfs_feed) -> None:
        """Test that stops are loaded correctly."""
        stops = sample_gtfs_feed.stops
        assert len(stops) == 5

        # Attributes are 'id', 'name', 'latitude', 'longitude'
        stop_ids = {s.id for s in stops}
        assert "stop_1" in stop_ids
        assert "stop_4" in stop_ids  # Parent station (Central Hub)
        assert "stop_5" in stop_ids  # Child station (Platform 1)

        # Check stop with coordinates
        stop_1 = next(s for s in stops if s.id == "stop_1")
        assert stop_1.name == "Main Street Station"
        assert abs(stop_1.latitude - 40.712776) < 0.0001
        assert abs(stop_1.longitude - (-74.005974)) < 0.0001

    def test_routes_loaded(self, sample_gtfs_feed) -> None:
        """Test that routes are loaded correctly."""
        routes = sample_gtfs_feed.routes
        assert len(routes) == 3

        # Attributes are 'id', 'short_name', 'long_name', 'route_type'
        route_ids = {r.id for r in routes}
        assert "route_1" in route_ids
        assert "route_2" in route_ids
        assert "route_3" in route_ids

        # Check route details
        route_1 = next(r for r in routes if r.id == "route_1")
        assert route_1.short_name == "1"
        assert route_1.long_name == "Main Line"
        assert route_1.route_type == 3  # Bus

    def test_trips_loaded(self, sample_gtfs_feed) -> None:
        """Test that trips are loaded correctly."""
        trips = sample_gtfs_feed.trips
        assert len(trips) == 5

        # Attributes are 'id', 'route_id', 'service_id', 'headsign'
        trip_ids = {t.id for t in trips}
        assert "trip_1" in trip_ids
        assert "trip_2" in trip_ids
        assert "trip_5" in trip_ids

        # Check trip details
        trip_1 = next(t for t in trips if t.id == "trip_1")
        assert trip_1.route_id == "route_1"
        assert trip_1.service_id == "weekday"
        assert trip_1.headsign == "Northbound to Central"

    def test_stop_times_loaded(self, sample_gtfs_feed) -> None:
        """Test that stop times are loaded correctly."""
        stop_times = sample_gtfs_feed.stop_times
        assert len(stop_times) == 16

        # Check stop times for trip_1
        trip_1_times = [st for st in stop_times if st.trip_id == "trip_1"]
        assert len(trip_1_times) == 4

        # Check sequence
        sequences = sorted(st.stop_sequence for st in trip_1_times)
        assert sequences == [1, 2, 3, 4]

    def test_calendars_loaded(self, sample_gtfs_feed) -> None:
        """Test that calendar entries are loaded correctly."""
        # Property is 'calendars', not 'calendar'
        calendars = sample_gtfs_feed.calendars
        assert len(calendars) == 2

        service_ids = {c.service_id for c in calendars}
        assert "weekday" in service_ids
        assert "weekend" in service_ids

        # Days are booleans (True/False), not integers
        weekday = next(c for c in calendars if c.service_id == "weekday")
        assert weekday.monday is True
        assert weekday.saturday is False
        assert weekday.sunday is False

    def test_calendar_dates_loaded(self, sample_gtfs_feed) -> None:
        """Test that calendar dates are loaded correctly."""
        calendar_dates = sample_gtfs_feed.calendar_dates
        assert len(calendar_dates) == 3

        # Check holiday exception (date format may vary)
        july_4th = [cd for cd in calendar_dates if "0704" in cd.date]
        assert len(july_4th) == 2

    def test_shapes_loaded(self, sample_gtfs_feed) -> None:
        """Test that shapes are loaded correctly."""
        shapes = sample_gtfs_feed.shapes
        # Shapes are aggregated objects with 'id' and 'points'
        assert len(shapes) >= 1

        shape_ids = {s.id for s in shapes}
        assert "shape_1" in shape_ids

    def test_count_properties(self, sample_gtfs_feed) -> None:
        """Test count properties for all entity types."""
        # These are properties (int attributes), not methods
        assert sample_gtfs_feed.agency_count == 2
        assert sample_gtfs_feed.stop_count == 5
        assert sample_gtfs_feed.route_count == 3
        assert sample_gtfs_feed.trip_count == 5
        assert sample_gtfs_feed.stop_time_count == 16


class TestGtfsFeedEdgeCases:
    """Tests for edge cases in GTFS loading."""

    def test_nonexistent_path_raises_error(self) -> None:
        """Test that loading from a nonexistent path raises an error."""
        from transit_parser import GtfsFeed

        with pytest.raises(Exception):
            GtfsFeed.from_path("/nonexistent/path")

    def test_empty_directory_raises_error(self, tmp_path: Path) -> None:
        """Test that loading from an empty directory raises an error."""
        from transit_parser import GtfsFeed

        empty_dir = tmp_path / "empty"
        empty_dir.mkdir()

        with pytest.raises(Exception):
            GtfsFeed.from_path(str(empty_dir))


class TestGtfsFeedCaching:
    """Tests for caching behavior in GtfsFeed."""

    def test_repeated_access_returns_same_data(self, sample_gtfs_feed) -> None:
        """Test that repeated property access returns consistent data."""
        agencies1 = sample_gtfs_feed.agencies
        agencies2 = sample_gtfs_feed.agencies

        # Should have same length
        assert len(agencies1) == len(agencies2)

        # Values should be equal
        ids1 = {a.id for a in agencies1}
        ids2 = {a.id for a in agencies2}
        assert ids1 == ids2
