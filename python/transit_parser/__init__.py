"""High-performance transit data parser with TXC to GTFS conversion.

This package provides tools for parsing and converting transit data formats:
- GTFS Static (CSV-based)
- TransXChange (TXC) - UK XML format
- Generic CSV and JSON

Example usage:

    # Parse a GTFS feed (eager - parses all files on load)
    from transit_parser import GtfsFeed
    feed = GtfsFeed.from_zip("path/to/gtfs.zip")
    print(f"Routes: {len(feed.routes)}")

    # Parse a GTFS feed (lazy - defers parsing until access)
    from transit_parser import LazyGtfsFeed
    feed = LazyGtfsFeed.from_path("path/to/gtfs/")
    print(f"Stop times: {feed.stop_time_count}")  # Fast count
    stop_times = feed.stop_times  # Parses stop_times.txt on first access

    # Filter GTFS data
    from transit_parser import GtfsFeed
    from transit_parser.filtering import GtfsFilter
    feed = GtfsFeed.from_path("path/to/gtfs/")
    filtered = GtfsFilter(feed)
    route_trips = filtered.trips_for_route("route_1")
    active_services = filtered.active_services_on("2025-07-04")

    # Get DataFrames (requires pandas)
    from transit_parser.dataframes import GtfsDataFrames
    dfs = GtfsDataFrames.from_path("path/to/gtfs/")
    print(dfs.stop_times.head())

    # Parse a TXC document
    from transit_parser import TxcDocument
    doc = TxcDocument.from_path("path/to/file.xml")
    print(f"Services: {doc.service_count}")

    # Convert TXC to GTFS
    from transit_parser import TxcToGtfsConverter
    converter = TxcToGtfsConverter()
    result = converter.convert(doc)
    result.feed.to_zip("output.zip")
"""

__version__ = "0.1.0"

# Import from Rust bindings
from transit_parser._core import (
    # Data models
    Agency,
    Calendar,
    CalendarDate,
    ConversionOptions,
    ConversionResult,
    ConversionStats,
    # CSV/JSON
    CsvDocument,
    DeadheadInferenceResult,
    # GTFS
    GtfsFeed,
    JsonDocument,
    LazyGtfsFeed,
    Route,
    # Schedule
    Schedule,
    ScheduleRow,
    Shape,
    Stop,
    StopTime,
    Trip,
    # TXC
    TxcDocument,
    # Adapters
    TxcToGtfsConverter,
    ValidationConfig,
    ValidationResult,
)

# Import exceptions
from transit_parser.exceptions import (
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

__all__ = [
    # Version
    "__version__",
    # Data models
    "Agency",
    "Stop",
    "Route",
    "Trip",
    "StopTime",
    "Calendar",
    "CalendarDate",
    "Shape",
    # GTFS
    "GtfsFeed",
    "LazyGtfsFeed",
    # TXC
    "TxcDocument",
    # CSV/JSON
    "CsvDocument",
    "JsonDocument",
    # Adapters
    "TxcToGtfsConverter",
    "ConversionOptions",
    "ConversionResult",
    "ConversionStats",
    # Exceptions
    "TransitParserError",
    "GtfsError",
    "GtfsFileNotFoundError",
    "GtfsValidationError",
    "GtfsParseError",
    "TxcError",
    "TxcFileNotFoundError",
    "TxcValidationError",
    "TxcParseError",
    "ConversionError",
    "MappingError",
    "CalendarConversionError",
    "FilterError",
    "InvalidDateError",
    # Schedule
    "Schedule",
    "ScheduleRow",
    "ValidationConfig",
    "ValidationResult",
    "DeadheadInferenceResult",
]
