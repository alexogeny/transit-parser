"""Filtering API for GTFS feeds.

This module provides high-level filtering capabilities for GTFS data,
allowing you to filter by route, agency, date, and other criteria.

Example usage:

    from transit_parser import GtfsFeed
    from transit_parser.filtering import GtfsFilter

    feed = GtfsFeed.from_path("path/to/gtfs/")
    filtered = GtfsFilter(feed)

    # Filter by route
    route_1_trips = filtered.trips_for_route("route_1")

    # Filter by agency
    agency_routes = filtered.routes_for_agency("agency_1")

    # Filter by date
    active_services = filtered.active_services_on("2025-07-04")

    # Get stop times for a trip
    times = filtered.stop_times_for_trip("trip_1")
"""

from __future__ import annotations

from datetime import date, datetime
from typing import TYPE_CHECKING

from transit_parser.exceptions import InvalidDateError

if TYPE_CHECKING:
    from transit_parser import (
        Agency,
        Calendar,
        GtfsFeed,
        LazyGtfsFeed,
        Route,
        Shape,
        Stop,
        StopTime,
        Trip,
    )


class GtfsFilter:
    """Provides filtering and querying capabilities for GTFS feeds.

    This class wraps a GtfsFeed or LazyGtfsFeed and provides convenient
    methods for filtering data by various criteria.

    Attributes:
        feed: The underlying GTFS feed to filter.
    """

    def __init__(self, feed: GtfsFeed | LazyGtfsFeed) -> None:
        """Initialize the filter with a GTFS feed.

        Args:
            feed: A GtfsFeed or LazyGtfsFeed instance to filter.
        """
        self._feed = feed
        # Build indexes on first access
        self._stop_index: dict[str, Stop] | None = None
        self._route_index: dict[str, Route] | None = None
        self._trip_index: dict[str, Trip] | None = None
        self._agency_index: dict[str, Agency] | None = None
        self._service_index: dict[str, Calendar] | None = None

    @property
    def feed(self) -> GtfsFeed | LazyGtfsFeed:
        """Return the underlying feed."""
        return self._feed

    # -------------------------------------------------------------------------
    # Index building (lazy)
    # -------------------------------------------------------------------------

    def _build_stop_index(self) -> dict[str, Stop]:
        if self._stop_index is None:
            self._stop_index = {s.id: s for s in self._feed.stops}
        return self._stop_index

    def _build_route_index(self) -> dict[str, Route]:
        if self._route_index is None:
            self._route_index = {r.id: r for r in self._feed.routes}
        return self._route_index

    def _build_trip_index(self) -> dict[str, Trip]:
        if self._trip_index is None:
            self._trip_index = {t.id: t for t in self._feed.trips}
        return self._trip_index

    def _build_agency_index(self) -> dict[str, Agency]:
        if self._agency_index is None:
            self._agency_index = {a.id: a for a in self._feed.agencies if a.id}
        return self._agency_index

    def _build_service_index(self) -> dict[str, Calendar]:
        if self._service_index is None:
            self._service_index = {c.service_id: c for c in self._feed.calendars}
        return self._service_index

    # -------------------------------------------------------------------------
    # Lookup by ID
    # -------------------------------------------------------------------------

    def get_stop(self, stop_id: str) -> Stop | None:
        """Get a stop by its ID.

        Args:
            stop_id: The stop ID to look up.

        Returns:
            The Stop object, or None if not found.
        """
        return self._build_stop_index().get(stop_id)

    def get_route(self, route_id: str) -> Route | None:
        """Get a route by its ID.

        Args:
            route_id: The route ID to look up.

        Returns:
            The Route object, or None if not found.
        """
        return self._build_route_index().get(route_id)

    def get_trip(self, trip_id: str) -> Trip | None:
        """Get a trip by its ID.

        Args:
            trip_id: The trip ID to look up.

        Returns:
            The Trip object, or None if not found.
        """
        return self._build_trip_index().get(trip_id)

    def get_agency(self, agency_id: str) -> Agency | None:
        """Get an agency by its ID.

        Args:
            agency_id: The agency ID to look up.

        Returns:
            The Agency object, or None if not found.
        """
        return self._build_agency_index().get(agency_id)

    def get_calendar(self, service_id: str) -> Calendar | None:
        """Get a calendar entry by service ID.

        Args:
            service_id: The service ID to look up.

        Returns:
            The Calendar object, or None if not found.
        """
        return self._build_service_index().get(service_id)

    # -------------------------------------------------------------------------
    # Filter by route
    # -------------------------------------------------------------------------

    def trips_for_route(self, route_id: str) -> list[Trip]:
        """Get all trips for a specific route.

        Args:
            route_id: The route ID to filter by.

        Returns:
            List of Trip objects for the specified route.
        """
        return [t for t in self._feed.trips if t.route_id == route_id]

    def stop_times_for_route(self, route_id: str) -> list[StopTime]:
        """Get all stop times for a specific route.

        Args:
            route_id: The route ID to filter by.

        Returns:
            List of StopTime objects for trips on the specified route.
        """
        trip_ids = {t.id for t in self.trips_for_route(route_id)}
        return [st for st in self._feed.stop_times if st.trip_id in trip_ids]

    def stops_for_route(self, route_id: str) -> list[Stop]:
        """Get all unique stops served by a specific route.

        Args:
            route_id: The route ID to filter by.

        Returns:
            List of unique Stop objects served by the route.
        """
        stop_ids = {st.stop_id for st in self.stop_times_for_route(route_id)}
        stop_index = self._build_stop_index()
        return [stop_index[sid] for sid in stop_ids if sid in stop_index]

    # -------------------------------------------------------------------------
    # Filter by agency
    # -------------------------------------------------------------------------

    def routes_for_agency(self, agency_id: str) -> list[Route]:
        """Get all routes operated by a specific agency.

        Args:
            agency_id: The agency ID to filter by.

        Returns:
            List of Route objects for the specified agency.
        """
        # Routes may have agency_id attribute or we need to infer from trips
        routes = []
        for route in self._feed.routes:
            # Check if route has agency_id attribute
            if hasattr(route, "agency_id"):
                if route.agency_id == agency_id:
                    routes.append(route)
            else:
                # If no agency_id on route, include it (single-agency feed)
                routes.append(route)
        return routes

    def trips_for_agency(self, agency_id: str) -> list[Trip]:
        """Get all trips for routes operated by a specific agency.

        Args:
            agency_id: The agency ID to filter by.

        Returns:
            List of Trip objects for routes operated by the agency.
        """
        route_ids = {r.id for r in self.routes_for_agency(agency_id)}
        return [t for t in self._feed.trips if t.route_id in route_ids]

    # -------------------------------------------------------------------------
    # Filter by trip
    # -------------------------------------------------------------------------

    def stop_times_for_trip(self, trip_id: str) -> list[StopTime]:
        """Get all stop times for a specific trip, ordered by sequence.

        Args:
            trip_id: The trip ID to filter by.

        Returns:
            List of StopTime objects for the trip, sorted by stop_sequence.
        """
        times = [st for st in self._feed.stop_times if st.trip_id == trip_id]
        return sorted(times, key=lambda st: st.stop_sequence)

    def stops_for_trip(self, trip_id: str) -> list[Stop]:
        """Get all stops for a specific trip, in sequence order.

        Args:
            trip_id: The trip ID to filter by.

        Returns:
            List of Stop objects for the trip, in stop sequence order.
        """
        stop_times = self.stop_times_for_trip(trip_id)
        stop_index = self._build_stop_index()
        return [stop_index[st.stop_id] for st in stop_times if st.stop_id in stop_index]

    # -------------------------------------------------------------------------
    # Filter by stop
    # -------------------------------------------------------------------------

    def stop_times_at_stop(self, stop_id: str) -> list[StopTime]:
        """Get all stop times at a specific stop.

        Args:
            stop_id: The stop ID to filter by.

        Returns:
            List of StopTime objects at the specified stop.
        """
        return [st for st in self._feed.stop_times if st.stop_id == stop_id]

    def trips_serving_stop(self, stop_id: str) -> list[Trip]:
        """Get all trips that serve a specific stop.

        Args:
            stop_id: The stop ID to filter by.

        Returns:
            List of Trip objects that stop at the specified stop.
        """
        trip_ids = {st.trip_id for st in self.stop_times_at_stop(stop_id)}
        trip_index = self._build_trip_index()
        return [trip_index[tid] for tid in trip_ids if tid in trip_index]

    def routes_serving_stop(self, stop_id: str) -> list[Route]:
        """Get all routes that serve a specific stop.

        Args:
            stop_id: The stop ID to filter by.

        Returns:
            List of Route objects that serve the specified stop.
        """
        route_ids = {t.route_id for t in self.trips_serving_stop(stop_id)}
        route_index = self._build_route_index()
        return [route_index[rid] for rid in route_ids if rid in route_index]

    # -------------------------------------------------------------------------
    # Filter by service/date
    # -------------------------------------------------------------------------

    def trips_for_service(self, service_id: str) -> list[Trip]:
        """Get all trips for a specific service.

        Args:
            service_id: The service ID to filter by.

        Returns:
            List of Trip objects using the specified service.
        """
        return [t for t in self._feed.trips if t.service_id == service_id]

    def _parse_date(self, date_input: str | date) -> date:
        """Parse a date input into a date object.

        Args:
            date_input: Date as string (YYYY-MM-DD or YYYYMMDD) or date object.

        Returns:
            A date object.

        Raises:
            InvalidDateError: If the date string cannot be parsed.
        """
        if isinstance(date_input, date):
            return date_input
        # Try common formats
        for fmt in ("%Y-%m-%d", "%Y%m%d"):
            try:
                return datetime.strptime(date_input, fmt).date()
            except ValueError:
                continue
        raise InvalidDateError(
            f"Cannot parse date: {date_input}",
            date_string=date_input,
            expected_format="YYYY-MM-DD or YYYYMMDD",
        )

    def _date_to_gtfs_format(self, d: date) -> str:
        """Convert a date to GTFS format (YYYYMMDD)."""
        return d.strftime("%Y%m%d")

    def active_services_on(self, date_input: str | date) -> list[Calendar]:
        """Get all services active on a specific date.

        Takes into account both the regular calendar and calendar_dates
        exceptions.

        Args:
            date_input: The date to check (string "YYYY-MM-DD" or date object).

        Returns:
            List of Calendar objects for services active on the date.
        """
        target_date = self._parse_date(date_input)
        target_str = self._date_to_gtfs_format(target_date)
        weekday = target_date.weekday()  # 0=Monday, 6=Sunday

        # Map weekday to calendar attribute
        day_attrs = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"]
        day_attr = day_attrs[weekday]

        active_services = []

        # Check regular calendar
        for cal in self._feed.calendars:
            # Check date range
            start = cal.start_date
            end = cal.end_date
            if start <= target_str <= end:
                # Check if service runs on this day of week
                if getattr(cal, day_attr):
                    active_services.append(cal)

        # Apply calendar_dates exceptions
        active_service_ids = {s.service_id for s in active_services}

        for cd in self._feed.calendar_dates:
            if cd.date == target_str or target_str in cd.date:
                if cd.exception_type == 1:  # Service added
                    # Add service if not already active
                    if cd.service_id not in active_service_ids:
                        cal = self.get_calendar(cd.service_id)
                        if cal:
                            active_services.append(cal)
                            active_service_ids.add(cd.service_id)
                elif cd.exception_type == 2:  # Service removed
                    # Remove service if active
                    active_services = [s for s in active_services if s.service_id != cd.service_id]
                    active_service_ids.discard(cd.service_id)

        return active_services

    def trips_on_date(self, date_input: str | date) -> list[Trip]:
        """Get all trips running on a specific date.

        Args:
            date_input: The date to check (string "YYYY-MM-DD" or date object).

        Returns:
            List of Trip objects running on the specified date.
        """
        active_service_ids = {s.service_id for s in self.active_services_on(date_input)}
        return [t for t in self._feed.trips if t.service_id in active_service_ids]

    # -------------------------------------------------------------------------
    # Shape queries
    # -------------------------------------------------------------------------

    def shape_for_trip(self, trip_id: str) -> Shape | None:
        """Get the shape for a specific trip.

        Args:
            trip_id: The trip ID to get the shape for.

        Returns:
            The Shape object for the trip, or None if no shape assigned.
        """
        trip = self.get_trip(trip_id)
        if not trip or not hasattr(trip, "shape_id") or not trip.shape_id:
            return None

        for shape in self._feed.shapes:
            if shape.id == trip.shape_id:
                return shape
        return None

    # -------------------------------------------------------------------------
    # Summary statistics
    # -------------------------------------------------------------------------

    def route_stop_count(self, route_id: str) -> int:
        """Get the number of unique stops served by a route.

        Args:
            route_id: The route ID to count stops for.

        Returns:
            Number of unique stops on the route.
        """
        return len(self.stops_for_route(route_id))

    def route_trip_count(self, route_id: str) -> int:
        """Get the number of trips for a route.

        Args:
            route_id: The route ID to count trips for.

        Returns:
            Number of trips on the route.
        """
        return len(self.trips_for_route(route_id))

    def stop_trip_count(self, stop_id: str) -> int:
        """Get the number of trips serving a stop.

        Args:
            stop_id: The stop ID to count trips for.

        Returns:
            Number of trips serving the stop.
        """
        return len({st.trip_id for st in self.stop_times_at_stop(stop_id)})
