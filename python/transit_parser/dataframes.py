"""Optional pandas DataFrame support for transit_parser.

This module provides DataFrame conversion methods for GTFS data.
Requires pandas to be installed.

Example usage:
    from transit_parser import LazyGtfsFeed
    from transit_parser.dataframes import to_dataframes

    feed = LazyGtfsFeed.from_path("gtfs/")
    dfs = to_dataframes(feed)
    print(dfs.stop_times.head())

Or using the convenience methods:
    from transit_parser.dataframes import GtfsDataFrames

    dfs = GtfsDataFrames.from_path("gtfs/")
    print(dfs.stop_times.head())
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import pandas as pd


def _check_pandas() -> Any:
    """Check if pandas is available."""
    try:
        import pandas
        return pandas
    except ImportError:
        raise ImportError(
            "pandas is required for DataFrame support. "
            "Install it with: pip install pandas"
        )


class GtfsDataFrames:
    """Lazy-loading GTFS DataFrames.

    Provides pandas DataFrame access to GTFS data with lazy loading.
    Each DataFrame is computed on first access and cached.
    """

    def __init__(self, feed: Any):
        """Create DataFrames wrapper from a GtfsFeed or LazyGtfsFeed.

        Args:
            feed: A GtfsFeed or LazyGtfsFeed instance
        """
        self._feed = feed
        self._agencies_df: pd.DataFrame | None = None
        self._stops_df: pd.DataFrame | None = None
        self._routes_df: pd.DataFrame | None = None
        self._trips_df: pd.DataFrame | None = None
        self._stop_times_df: pd.DataFrame | None = None
        self._calendars_df: pd.DataFrame | None = None
        self._calendar_dates_df: pd.DataFrame | None = None
        self._shapes_df: pd.DataFrame | None = None

    @classmethod
    def from_path(cls, path: str) -> GtfsDataFrames:
        """Load GTFS feed from a directory path as DataFrames.

        Args:
            path: Path to GTFS directory

        Returns:
            GtfsDataFrames instance with lazy-loaded DataFrames
        """
        from transit_parser import LazyGtfsFeed
        feed = LazyGtfsFeed.from_path(path)
        return cls(feed)

    @classmethod
    def from_zip(cls, path: str) -> GtfsDataFrames:
        """Load GTFS feed from a ZIP file as DataFrames.

        Args:
            path: Path to GTFS ZIP file

        Returns:
            GtfsDataFrames instance with lazy-loaded DataFrames
        """
        from transit_parser import LazyGtfsFeed
        feed = LazyGtfsFeed.from_zip(path)
        return cls(feed)

    @property
    def agencies(self) -> pd.DataFrame:
        """Get agencies as a DataFrame."""
        if self._agencies_df is None:
            pd = _check_pandas()
            agencies = self._feed.agencies
            self._agencies_df = pd.DataFrame([
                {
                    "agency_id": a.id,
                    "agency_name": a.name,
                    "agency_url": a.url,
                    "agency_timezone": a.timezone,
                }
                for a in agencies
            ])
        return self._agencies_df

    @property
    def stops(self) -> pd.DataFrame:
        """Get stops as a DataFrame."""
        if self._stops_df is None:
            pd = _check_pandas()
            stops = self._feed.stops
            self._stops_df = pd.DataFrame([
                {
                    "stop_id": s.id,
                    "stop_code": s.code,
                    "stop_name": s.name,
                    "stop_lat": s.latitude,
                    "stop_lon": s.longitude,
                }
                for s in stops
            ])
        return self._stops_df

    @property
    def routes(self) -> pd.DataFrame:
        """Get routes as a DataFrame."""
        if self._routes_df is None:
            pd = _check_pandas()
            routes = self._feed.routes
            self._routes_df = pd.DataFrame([
                {
                    "route_id": r.id,
                    "route_short_name": r.short_name,
                    "route_long_name": r.long_name,
                    "route_type": r.route_type,
                }
                for r in routes
            ])
        return self._routes_df

    @property
    def trips(self) -> pd.DataFrame:
        """Get trips as a DataFrame."""
        if self._trips_df is None:
            pd = _check_pandas()
            trips = self._feed.trips
            self._trips_df = pd.DataFrame([
                {
                    "trip_id": t.id,
                    "route_id": t.route_id,
                    "service_id": t.service_id,
                    "trip_headsign": t.headsign,
                }
                for t in trips
            ])
        return self._trips_df

    @property
    def stop_times(self) -> pd.DataFrame:
        """Get stop_times as a DataFrame."""
        if self._stop_times_df is None:
            pd = _check_pandas()
            stop_times = self._feed.stop_times
            self._stop_times_df = pd.DataFrame([
                {
                    "trip_id": st.trip_id,
                    "arrival_time": st.arrival_time,
                    "departure_time": st.departure_time,
                    "stop_id": st.stop_id,
                    "stop_sequence": st.stop_sequence,
                }
                for st in stop_times
            ])
        return self._stop_times_df

    @property
    def calendar(self) -> pd.DataFrame:
        """Get calendar as a DataFrame."""
        if self._calendars_df is None:
            pd = _check_pandas()
            calendars = self._feed.calendars
            self._calendars_df = pd.DataFrame([
                {
                    "service_id": c.service_id,
                    "monday": int(c.monday),
                    "tuesday": int(c.tuesday),
                    "wednesday": int(c.wednesday),
                    "thursday": int(c.thursday),
                    "friday": int(c.friday),
                    "saturday": int(c.saturday),
                    "sunday": int(c.sunday),
                    "start_date": c.start_date,
                    "end_date": c.end_date,
                }
                for c in calendars
            ])
        return self._calendars_df

    @property
    def calendar_dates(self) -> pd.DataFrame:
        """Get calendar_dates as a DataFrame."""
        if self._calendar_dates_df is None:
            pd = _check_pandas()
            dates = self._feed.calendar_dates
            self._calendar_dates_df = pd.DataFrame([
                {
                    "service_id": d.service_id,
                    "date": d.date,
                    "exception_type": d.exception_type,
                }
                for d in dates
            ])
        return self._calendar_dates_df

    @property
    def shapes(self) -> pd.DataFrame:
        """Get shapes as a DataFrame (flattened to one row per point)."""
        if self._shapes_df is None:
            pd = _check_pandas()
            shapes = self._feed.shapes
            rows = []
            for shape in shapes:
                for lat, lon, seq in shape.points:
                    rows.append({
                        "shape_id": shape.id,
                        "shape_pt_lat": lat,
                        "shape_pt_lon": lon,
                        "shape_pt_sequence": seq,
                    })
            self._shapes_df = pd.DataFrame(rows)
        return self._shapes_df


def to_dataframes(feed: Any) -> GtfsDataFrames:
    """Convert a GTFS feed to DataFrames.

    Args:
        feed: A GtfsFeed or LazyGtfsFeed instance

    Returns:
        GtfsDataFrames instance with lazy-loaded DataFrames

    Example:
        >>> from transit_parser import GtfsFeed
        >>> from transit_parser.dataframes import to_dataframes
        >>> feed = GtfsFeed.from_path("gtfs/")
        >>> dfs = to_dataframes(feed)
        >>> print(dfs.stop_times.head())
    """
    return GtfsDataFrames(feed)
