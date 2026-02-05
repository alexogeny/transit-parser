"""Type stubs for the Rust extension module."""


# Data Models

class Agency:
    """A transit agency."""

    def __init__(
        self,
        name: str,
        url: str,
        timezone: str,
        id: str | None = None,
    ) -> None: ...

    @property
    def id(self) -> str | None: ...
    @property
    def name(self) -> str: ...
    @property
    def url(self) -> str: ...
    @property
    def timezone(self) -> str: ...

class Stop:
    """A transit stop."""

    def __init__(
        self,
        id: str,
        name: str,
        latitude: float,
        longitude: float,
    ) -> None: ...

    @property
    def id(self) -> str: ...
    @property
    def name(self) -> str: ...
    @property
    def latitude(self) -> float: ...
    @property
    def longitude(self) -> float: ...
    @property
    def code(self) -> str | None: ...

class Route:
    """A transit route."""

    @property
    def id(self) -> str: ...
    @property
    def short_name(self) -> str | None: ...
    @property
    def long_name(self) -> str | None: ...
    @property
    def route_type(self) -> int: ...

class Trip:
    """A transit trip."""

    @property
    def id(self) -> str: ...
    @property
    def route_id(self) -> str: ...
    @property
    def service_id(self) -> str: ...
    @property
    def headsign(self) -> str | None: ...

class StopTime:
    """A stop time within a trip."""

    @property
    def trip_id(self) -> str: ...
    @property
    def stop_id(self) -> str: ...
    @property
    def stop_sequence(self) -> int: ...
    @property
    def arrival_time(self) -> str | None: ...
    @property
    def departure_time(self) -> str | None: ...

class Calendar:
    """A service calendar."""

    @property
    def service_id(self) -> str: ...
    @property
    def monday(self) -> bool: ...
    @property
    def tuesday(self) -> bool: ...
    @property
    def wednesday(self) -> bool: ...
    @property
    def thursday(self) -> bool: ...
    @property
    def friday(self) -> bool: ...
    @property
    def saturday(self) -> bool: ...
    @property
    def sunday(self) -> bool: ...
    @property
    def start_date(self) -> str: ...
    @property
    def end_date(self) -> str: ...

class CalendarDate:
    """A calendar exception date."""

    @property
    def service_id(self) -> str: ...
    @property
    def date(self) -> str: ...
    @property
    def exception_type(self) -> int: ...

class Shape:
    """A route shape."""

    @property
    def id(self) -> str: ...
    @property
    def points(self) -> list[tuple[float, float, int]]: ...

# GTFS

class GtfsFeed:
    """A GTFS feed."""

    def __init__(self) -> None: ...

    @staticmethod
    def from_path(path: str) -> GtfsFeed: ...

    @staticmethod
    def from_zip(path: str) -> GtfsFeed: ...

    @staticmethod
    def from_bytes(data: bytes) -> GtfsFeed: ...

    def to_path(self, path: str) -> None: ...
    def to_zip(self, path: str) -> None: ...
    def to_bytes(self) -> bytes: ...

    @property
    def agencies(self) -> list[Agency]: ...
    @property
    def stops(self) -> list[Stop]: ...
    @property
    def routes(self) -> list[Route]: ...
    @property
    def trips(self) -> list[Trip]: ...
    @property
    def stop_times(self) -> list[StopTime]: ...
    @property
    def calendars(self) -> list[Calendar]: ...
    @property
    def calendar_dates(self) -> list[CalendarDate]: ...
    @property
    def shapes(self) -> list[Shape]: ...

    @property
    def agency_count(self) -> int: ...
    @property
    def stop_count(self) -> int: ...
    @property
    def route_count(self) -> int: ...
    @property
    def trip_count(self) -> int: ...
    @property
    def stop_time_count(self) -> int: ...
    @property
    def calendar_count(self) -> int: ...
    @property
    def calendar_date_count(self) -> int: ...
    @property
    def shape_count(self) -> int: ...


class LazyGtfsFeed:
    """A lazily-loaded GTFS feed.

    This class defers CSV parsing until properties are first accessed,
    providing faster initial load times for large feeds.
    """

    def __init__(self) -> None: ...

    @staticmethod
    def from_path(path: str) -> LazyGtfsFeed:
        """Load a GTFS feed lazily from a directory path."""
        ...

    @staticmethod
    def from_zip(path: str) -> LazyGtfsFeed:
        """Load a GTFS feed lazily from a ZIP file."""
        ...

    @staticmethod
    def from_bytes(data: bytes) -> LazyGtfsFeed:
        """Load a GTFS feed lazily from ZIP bytes."""
        ...

    def materialize(self) -> GtfsFeed:
        """Convert to a regular GtfsFeed with all data loaded."""
        ...

    @property
    def agencies(self) -> list[Agency]: ...
    @property
    def stops(self) -> list[Stop]: ...
    @property
    def routes(self) -> list[Route]: ...
    @property
    def trips(self) -> list[Trip]: ...
    @property
    def stop_times(self) -> list[StopTime]: ...
    @property
    def calendars(self) -> list[Calendar]: ...
    @property
    def calendar_dates(self) -> list[CalendarDate]: ...
    @property
    def shapes(self) -> list[Shape]: ...

    @property
    def agency_count(self) -> int: ...
    @property
    def stop_count(self) -> int: ...
    @property
    def route_count(self) -> int: ...
    @property
    def trip_count(self) -> int: ...
    @property
    def stop_time_count(self) -> int: ...
    @property
    def calendar_count(self) -> int: ...
    @property
    def calendar_date_count(self) -> int: ...
    @property
    def shape_count(self) -> int: ...


# TXC

class TxcDocument:
    """A TransXChange document."""

    @staticmethod
    def from_path(path: str) -> TxcDocument: ...

    @staticmethod
    def from_bytes(data: bytes) -> TxcDocument: ...

    @staticmethod
    def from_string(xml: str) -> TxcDocument: ...

    @property
    def schema_version(self) -> str: ...
    @property
    def filename(self) -> str | None: ...
    @property
    def operator_count(self) -> int: ...
    @property
    def service_count(self) -> int: ...
    @property
    def stop_point_count(self) -> int: ...
    @property
    def vehicle_journey_count(self) -> int: ...
    @property
    def journey_pattern_section_count(self) -> int: ...

    def get_operator_names(self) -> list[str]: ...
    def get_service_codes(self) -> list[str]: ...
    def get_stop_codes(self) -> list[str]: ...

# CSV

class CsvDocument:
    """A CSV document."""

    def __init__(self) -> None: ...

    @staticmethod
    def from_path(path: str) -> CsvDocument: ...

    @staticmethod
    def from_bytes(data: bytes) -> CsvDocument: ...

    @staticmethod
    def from_string(csv: str) -> CsvDocument: ...

    def to_path(self, path: str) -> None: ...
    def to_string(self) -> str: ...

    def __len__(self) -> int: ...

    @property
    def columns(self) -> list[str]: ...
    @property
    def rows(self) -> list[dict]: ...

# JSON

class JsonDocument:
    """A JSON document."""

    @staticmethod
    def from_path(path: str) -> JsonDocument: ...

    @staticmethod
    def from_bytes(data: bytes) -> JsonDocument: ...

    @staticmethod
    def from_string(json: str) -> JsonDocument: ...

    def to_path(self, path: str) -> None: ...
    def to_string(self) -> str: ...
    def to_string_pretty(self) -> str: ...

    def is_object(self) -> bool: ...
    def is_array(self) -> bool: ...

    @property
    def root(self) -> object: ...

    def pointer(self, path: str) -> object | None: ...

# Adapters

class ConversionOptions:
    """Options for TXC to GTFS conversion."""

    def __init__(
        self,
        include_shapes: bool = False,
        calendar_start: str | None = None,
        calendar_end: str | None = None,
        region: str = "england",
        default_timezone: str = "Europe/London",
        default_agency_url: str = "https://example.com",
    ) -> None: ...

    @property
    def include_shapes(self) -> bool: ...

class ConversionStats:
    """Statistics from conversion."""

    @property
    def agencies_converted(self) -> int: ...
    @property
    def stops_converted(self) -> int: ...
    @property
    def routes_converted(self) -> int: ...
    @property
    def trips_converted(self) -> int: ...
    @property
    def stop_times_generated(self) -> int: ...
    @property
    def calendar_entries(self) -> int: ...
    @property
    def calendar_exceptions(self) -> int: ...
    @property
    def shapes_generated(self) -> int: ...

class ConversionResult:
    """Result of TXC to GTFS conversion."""

    @property
    def feed(self) -> GtfsFeed: ...
    @property
    def warnings(self) -> list[str]: ...
    @property
    def stats(self) -> ConversionStats: ...

class TxcToGtfsConverter:
    """TXC to GTFS converter."""

    def __init__(self, options: ConversionOptions | None = None) -> None: ...

    def convert(self, document: TxcDocument) -> ConversionResult: ...
    def convert_batch(self, documents: list[TxcDocument]) -> ConversionResult: ...

# Schedule

class ScheduleRow:
    """A single row in a schedule file.

    Represents one movement or activity: a revenue trip, deadhead,
    break, or relief.
    """

    def __init__(self) -> None: ...

    @property
    def run_number(self) -> str | None: ...
    @run_number.setter
    def run_number(self, value: str | None) -> None: ...

    @property
    def block(self) -> str | None: ...
    @block.setter
    def block(self, value: str | None) -> None: ...

    @property
    def start_place(self) -> str | None: ...
    @start_place.setter
    def start_place(self, value: str | None) -> None: ...

    @property
    def end_place(self) -> str | None: ...
    @end_place.setter
    def end_place(self, value: str | None) -> None: ...

    @property
    def start_time(self) -> str | None: ...
    @start_time.setter
    def start_time(self, value: str | None) -> None: ...

    @property
    def end_time(self) -> str | None: ...
    @end_time.setter
    def end_time(self, value: str | None) -> None: ...

    @property
    def trip_id(self) -> str | None: ...
    @trip_id.setter
    def trip_id(self, value: str | None) -> None: ...

    @property
    def depot(self) -> str | None: ...
    @depot.setter
    def depot(self, value: str | None) -> None: ...

    @property
    def vehicle_class(self) -> str | None: ...
    @property
    def vehicle_type(self) -> str | None: ...
    @property
    def start_lat(self) -> float | None: ...
    @property
    def start_lon(self) -> float | None: ...
    @property
    def end_lat(self) -> float | None: ...
    @property
    def end_lon(self) -> float | None: ...
    @property
    def route_shape_id(self) -> str | None: ...
    @property
    def row_type(self) -> str: ...
    @property
    def duty_id(self) -> str | None: ...
    @property
    def shift_id(self) -> str | None: ...

    def is_revenue(self) -> bool:
        """Check if this is a revenue (passenger-carrying) trip."""
        ...

    def is_deadhead(self) -> bool:
        """Check if this is a deadhead movement."""
        ...

    def duration_seconds(self) -> int | None:
        """Get duration in seconds."""
        ...


class Schedule:
    """A transit schedule containing rows, blocks, and duties."""

    def __init__(self) -> None: ...

    @staticmethod
    def from_csv(path: str) -> Schedule:
        """Load a schedule from a CSV file."""
        ...

    @staticmethod
    def from_csv_string(csv_str: str) -> Schedule:
        """Load a schedule from a CSV string."""
        ...

    @staticmethod
    def from_csv_with_mapping(
        path: str,
        column_mapping: dict[str, str] | None = None,
    ) -> Schedule:
        """Load a schedule with custom column mapping."""
        ...

    def __len__(self) -> int: ...

    @property
    def rows(self) -> list[ScheduleRow]: ...

    @property
    def revenue_trip_count(self) -> int: ...

    def block_ids(self) -> list[str]:
        """Get unique block IDs."""
        ...

    def run_numbers(self) -> list[str]:
        """Get unique run numbers."""
        ...

    def depots(self) -> list[str]:
        """Get unique depot codes."""
        ...

    def trip_ids(self) -> list[str]:
        """Get unique trip IDs."""
        ...

    def summary(self) -> dict[str, int]:
        """Get summary statistics."""
        ...

    def validate(
        self,
        gtfs: GtfsFeed,
        config: ValidationConfig | None = None,
    ) -> ValidationResult:
        """Validate the schedule against GTFS data."""
        ...

    def validate_structure(
        self,
        config: ValidationConfig | None = None,
    ) -> ValidationResult:
        """Validate schedule structure (without GTFS)."""
        ...

    def infer_deadheads(
        self,
        gtfs: GtfsFeed | None = None,
        default_depot: str | None = None,
    ) -> DeadheadInferenceResult:
        """Infer missing deadheads."""
        ...

    def to_csv(
        self,
        path: str,
        columns: list[str] | None = None,
        preset: str | None = None,
    ) -> None:
        """Export to CSV file.

        Args:
            path: Output file path.
            columns: Custom column list to export.
            preset: Export preset name (default, minimal, extended,
                    optibus, hastus, gtfs_block).
        """
        ...

    def to_csv_string(
        self,
        columns: list[str] | None = None,
        preset: str | None = None,
    ) -> str:
        """Export to CSV string."""
        ...


class ValidationConfig:
    """Configuration for schedule validation."""

    def __init__(
        self,
        gtfs_compliance: str | None = None,
        min_layover_seconds: int | None = None,
        max_trip_duration_seconds: int | None = None,
        max_duty_length_seconds: int | None = None,
        max_continuous_driving_seconds: int | None = None,
        min_break_duration_seconds: int | None = None,
        time_tolerance_seconds: int | None = None,
        validate_block_continuity: bool | None = None,
        validate_duty_constraints: bool | None = None,
        generate_warnings: bool | None = None,
    ) -> None:
        """Create validation config.

        Args:
            gtfs_compliance: Compliance level (strict, standard, lenient).
            min_layover_seconds: Minimum time between trips (default: 300).
            max_trip_duration_seconds: Maximum trip duration (default: 14400).
            max_duty_length_seconds: Maximum duty length (default: 32400).
            max_continuous_driving_seconds: Max driving before break (default: 16200).
            min_break_duration_seconds: Minimum break length (default: 1800).
            time_tolerance_seconds: Allowed deviation from GTFS times (default: 60).
            validate_block_continuity: Whether to validate block continuity.
            validate_duty_constraints: Whether to validate duty constraints.
            generate_warnings: Whether to generate warnings.
        """
        ...

    @staticmethod
    def strict() -> ValidationConfig:
        """Create a strict validation config."""
        ...

    @staticmethod
    def lenient() -> ValidationConfig:
        """Create a lenient validation config."""
        ...


class ValidationResult:
    """Result of schedule validation."""

    @property
    def is_valid(self) -> bool:
        """Check if validation passed (no errors)."""
        ...

    @property
    def error_count(self) -> int:
        """Get the number of errors."""
        ...

    @property
    def warning_count(self) -> int:
        """Get the number of warnings."""
        ...

    @property
    def errors(self) -> list[dict[str, str]]:
        """Get all errors as dicts with code, category, message, context."""
        ...

    @property
    def warnings(self) -> list[dict[str, str]]:
        """Get all warnings as dicts with code, category, message, context."""
        ...

    @property
    def rows_validated(self) -> int:
        """Get number of rows validated."""
        ...

    @property
    def blocks_validated(self) -> int:
        """Get number of blocks validated."""
        ...


class DeadheadInferenceResult:
    """Result of deadhead inference."""

    @property
    def pull_out_count(self) -> int:
        """Number of inferred pull-outs."""
        ...

    @property
    def pull_in_count(self) -> int:
        """Number of inferred pull-ins."""
        ...

    @property
    def interlining_count(self) -> int:
        """Number of inferred interlinings."""
        ...

    @property
    def total_count(self) -> int:
        """Total count of inferred deadheads."""
        ...

    @property
    def incomplete_blocks(self) -> list[str]:
        """Blocks that couldn't have deadheads inferred."""
        ...
