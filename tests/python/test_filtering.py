"""Unit tests for the filtering API."""

from __future__ import annotations

from datetime import date
from pathlib import Path

import pytest


class TestGtfsFilterLookup:
    """Tests for ID lookup methods."""

    def test_get_stop(self, sample_gtfs_feed) -> None:
        """Test looking up a stop by ID."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        stop = f.get_stop("stop_1")
        assert stop is not None
        assert stop.id == "stop_1"
        assert stop.name == "Main Street Station"

    def test_get_stop_not_found(self, sample_gtfs_feed) -> None:
        """Test looking up a nonexistent stop."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        stop = f.get_stop("nonexistent")
        assert stop is None

    def test_get_route(self, sample_gtfs_feed) -> None:
        """Test looking up a route by ID."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        route = f.get_route("route_1")
        assert route is not None
        assert route.id == "route_1"

    def test_get_trip(self, sample_gtfs_feed) -> None:
        """Test looking up a trip by ID."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        trip = f.get_trip("trip_1")
        assert trip is not None
        assert trip.id == "trip_1"

    def test_get_calendar(self, sample_gtfs_feed) -> None:
        """Test looking up a calendar by service ID."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        cal = f.get_calendar("weekday")
        assert cal is not None
        assert cal.service_id == "weekday"


class TestGtfsFilterByRoute:
    """Tests for filtering by route."""

    def test_trips_for_route(self, sample_gtfs_feed) -> None:
        """Test getting trips for a route."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        trips = f.trips_for_route("route_1")
        assert len(trips) >= 1
        assert all(t.route_id == "route_1" for t in trips)

    def test_stop_times_for_route(self, sample_gtfs_feed) -> None:
        """Test getting stop times for a route."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        stop_times = f.stop_times_for_route("route_1")
        assert len(stop_times) >= 1

    def test_stops_for_route(self, sample_gtfs_feed) -> None:
        """Test getting stops for a route."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        stops = f.stops_for_route("route_1")
        assert len(stops) >= 1

    def test_route_stop_count(self, sample_gtfs_feed) -> None:
        """Test counting stops for a route."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        count = f.route_stop_count("route_1")
        assert count >= 1

    def test_route_trip_count(self, sample_gtfs_feed) -> None:
        """Test counting trips for a route."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        count = f.route_trip_count("route_1")
        assert count >= 1


class TestGtfsFilterByTrip:
    """Tests for filtering by trip."""

    def test_stop_times_for_trip(self, sample_gtfs_feed) -> None:
        """Test getting stop times for a trip."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        stop_times = f.stop_times_for_trip("trip_1")
        assert len(stop_times) == 4  # trip_1 has 4 stops

        # Should be sorted by sequence
        sequences = [st.stop_sequence for st in stop_times]
        assert sequences == sorted(sequences)

    def test_stops_for_trip(self, sample_gtfs_feed) -> None:
        """Test getting stops for a trip in order."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        stops = f.stops_for_trip("trip_1")
        assert len(stops) == 4


class TestGtfsFilterByStop:
    """Tests for filtering by stop."""

    def test_stop_times_at_stop(self, sample_gtfs_feed) -> None:
        """Test getting stop times at a specific stop."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        stop_times = f.stop_times_at_stop("stop_1")
        assert len(stop_times) >= 1
        assert all(st.stop_id == "stop_1" for st in stop_times)

    def test_trips_serving_stop(self, sample_gtfs_feed) -> None:
        """Test getting trips that serve a stop."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        trips = f.trips_serving_stop("stop_1")
        assert len(trips) >= 1

    def test_routes_serving_stop(self, sample_gtfs_feed) -> None:
        """Test getting routes that serve a stop."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        routes = f.routes_serving_stop("stop_1")
        assert len(routes) >= 1

    def test_stop_trip_count(self, sample_gtfs_feed) -> None:
        """Test counting trips serving a stop."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        count = f.stop_trip_count("stop_1")
        assert count >= 1


class TestGtfsFilterByService:
    """Tests for filtering by service and date."""

    def test_trips_for_service(self, sample_gtfs_feed) -> None:
        """Test getting trips for a service."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        trips = f.trips_for_service("weekday")
        assert len(trips) >= 1
        assert all(t.service_id == "weekday" for t in trips)

    def test_active_services_on_weekday(self, sample_gtfs_feed) -> None:
        """Test getting active services on a weekday."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        # January 6, 2025 is a Monday
        services = f.active_services_on("2025-01-06")
        service_ids = {s.service_id for s in services}
        assert "weekday" in service_ids
        assert "weekend" not in service_ids

    def test_active_services_on_weekend(self, sample_gtfs_feed) -> None:
        """Test getting active services on a weekend."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        # January 4, 2025 is a Saturday
        services = f.active_services_on("2025-01-04")
        service_ids = {s.service_id for s in services}
        assert "weekend" in service_ids
        assert "weekday" not in service_ids

    def test_active_services_with_date_object(self, sample_gtfs_feed) -> None:
        """Test getting active services with a date object."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        # January 6, 2025 is a Monday
        services = f.active_services_on(date(2025, 1, 6))
        service_ids = {s.service_id for s in services}
        assert "weekday" in service_ids

    def test_trips_on_date(self, sample_gtfs_feed) -> None:
        """Test getting trips on a specific date."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)
        # January 6, 2025 is a Monday
        trips = f.trips_on_date("2025-01-06")
        assert len(trips) >= 1
        # All trips should be weekday service
        assert all(t.service_id == "weekday" for t in trips)


class TestGtfsFilterWithLazyFeed:
    """Tests for filtering with lazy GTFS feed."""

    def test_filter_with_lazy_feed(self, sample_lazy_gtfs_feed) -> None:
        """Test that filtering works with LazyGtfsFeed."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_lazy_gtfs_feed)

        # Basic lookups should work
        stop = f.get_stop("stop_1")
        assert stop is not None

        route = f.get_route("route_1")
        assert route is not None

        # Filtering should work
        trips = f.trips_for_route("route_1")
        assert len(trips) >= 1


class TestGtfsFilterIndexCaching:
    """Tests for index caching behavior."""

    def test_indexes_are_cached(self, sample_gtfs_feed) -> None:
        """Test that indexes are built once and cached."""
        from transit_parser.filtering import GtfsFilter

        f = GtfsFilter(sample_gtfs_feed)

        # First access builds the index
        stop1 = f.get_stop("stop_1")

        # Index should be cached now
        assert f._stop_index is not None

        # Second access should use cache
        stop2 = f.get_stop("stop_1")
        assert stop1.id == stop2.id
