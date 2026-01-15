# GTFS API Reference

## GtfsFeed

::: transit_parser.GtfsFeed
    options:
      show_source: false

### Loading Methods

#### from_path
```python
GtfsFeed.from_path(path: str) -> GtfsFeed
```
Load a GTFS feed from a directory containing CSV files.

#### from_zip
```python
GtfsFeed.from_zip(path: str) -> GtfsFeed
```
Load a GTFS feed from a ZIP archive.

#### from_bytes
```python
GtfsFeed.from_bytes(data: bytes) -> GtfsFeed
```
Load a GTFS feed from ZIP file bytes.

### Writing Methods

#### to_path
```python
feed.to_path(path: str) -> None
```
Write the feed to a directory as CSV files.

#### to_zip
```python
feed.to_zip(path: str) -> None
```
Write the feed to a ZIP archive.

#### to_bytes
```python
feed.to_bytes() -> bytes
```
Serialize the feed to ZIP bytes.

### Data Properties

| Property | Type | Description |
|----------|------|-------------|
| `agencies` | `List[Agency]` | All agencies |
| `stops` | `List[Stop]` | All stops |
| `routes` | `List[Route]` | All routes |
| `trips` | `List[Trip]` | All trips |
| `stop_times` | `List[StopTime]` | All stop times |
| `calendars` | `List[Calendar]` | Calendar entries |
| `calendar_dates` | `List[CalendarDate]` | Calendar exceptions |
| `shapes` | `List[Shape]` | Route shapes |

### Count Properties

| Property | Type | Description |
|----------|------|-------------|
| `agency_count` | `int` | Number of agencies |
| `stop_count` | `int` | Number of stops |
| `route_count` | `int` | Number of routes |
| `trip_count` | `int` | Number of trips |
| `stop_time_count` | `int` | Number of stop times |
| `calendar_count` | `int` | Number of calendar entries |
| `calendar_date_count` | `int` | Number of calendar exceptions |
| `shape_count` | `int` | Number of shapes |

---

## LazyGtfsFeed

A lazily-loaded GTFS feed that defers CSV parsing until first access.

### Loading Methods

Same as `GtfsFeed`:
- `from_path(path: str) -> LazyGtfsFeed`
- `from_zip(path: str) -> LazyGtfsFeed`
- `from_bytes(data: bytes) -> LazyGtfsFeed`

### Special Methods

#### materialize
```python
lazy_feed.materialize() -> GtfsFeed
```
Convert to a regular `GtfsFeed` with all data loaded.

### Properties

Same as `GtfsFeed` - all data properties and count properties are available.

**Key difference:** Data properties parse on first access, count properties are always fast.

---

## Data Models

### Agency

| Property | Type | Description |
|----------|------|-------------|
| `id` | `Optional[str]` | Agency ID |
| `name` | `str` | Agency name |
| `url` | `str` | Agency URL |
| `timezone` | `str` | Agency timezone |

### Stop

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Stop ID |
| `name` | `str` | Stop name |
| `code` | `Optional[str]` | Stop code |
| `latitude` | `float` | Latitude |
| `longitude` | `float` | Longitude |

### Route

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Route ID |
| `short_name` | `Optional[str]` | Short name (e.g., "1", "A") |
| `long_name` | `Optional[str]` | Long name |
| `route_type` | `int` | GTFS route type |

Route types: 0=tram, 1=subway, 2=rail, 3=bus, 4=ferry, 5=cable car, 6=gondola, 7=funicular

### Trip

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Trip ID |
| `route_id` | `str` | Parent route ID |
| `service_id` | `str` | Service ID |
| `headsign` | `Optional[str]` | Trip headsign |

### StopTime

| Property | Type | Description |
|----------|------|-------------|
| `trip_id` | `str` | Parent trip ID |
| `stop_id` | `str` | Stop ID |
| `stop_sequence` | `int` | Sequence number |
| `arrival_time` | `Optional[str]` | Arrival time (HH:MM:SS) |
| `departure_time` | `Optional[str]` | Departure time (HH:MM:SS) |

### Calendar

| Property | Type | Description |
|----------|------|-------------|
| `service_id` | `str` | Service ID |
| `monday` | `bool` | Runs on Monday |
| `tuesday` | `bool` | Runs on Tuesday |
| `wednesday` | `bool` | Runs on Wednesday |
| `thursday` | `bool` | Runs on Thursday |
| `friday` | `bool` | Runs on Friday |
| `saturday` | `bool` | Runs on Saturday |
| `sunday` | `bool` | Runs on Sunday |
| `start_date` | `str` | Start date (YYYYMMDD) |
| `end_date` | `str` | End date (YYYYMMDD) |

### CalendarDate

| Property | Type | Description |
|----------|------|-------------|
| `service_id` | `str` | Service ID |
| `date` | `str` | Date (YYYYMMDD) |
| `exception_type` | `int` | 1=added, 2=removed |

### Shape

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Shape ID |
| `points` | `List[Tuple[float, float, int]]` | (lat, lon, sequence) points |
