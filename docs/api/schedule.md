# Schedule API Reference

The schedule module provides classes for loading, validating, and exporting transit schedules.

## Schedule

```python
from transit_parser import Schedule
```

A transit schedule containing rows, blocks, and duties.

### Constructors

#### `Schedule()`

Create an empty schedule.

```python
schedule = Schedule()
```

#### `Schedule.from_csv(path)`

Load a schedule from a CSV file with automatic column detection.

```python
schedule = Schedule.from_csv("schedule.csv")
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `str` | Path to the CSV file |

**Returns:** `Schedule`

**Raises:** `IOError` if file cannot be read

#### `Schedule.from_csv_string(csv_str)`

Load a schedule from a CSV string.

```python
schedule = Schedule.from_csv_string("block,trip_id,start_time\nB1,T1,08:00:00")
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `csv_str` | `str` | CSV content as string |

**Returns:** `Schedule`

#### `Schedule.from_csv_with_mapping(path, column_mapping=None)`

Load a schedule with custom column mapping.

```python
schedule = Schedule.from_csv_with_mapping(
    "schedule.csv",
    column_mapping={
        "block": "vehicle_id",
        "trip_id": "journey_ref",
    }
)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `str` | Path to CSV file |
| `column_mapping` | `dict[str, str] \| None` | Maps field names to column names |

**Returns:** `Schedule`

### Properties

#### `rows`

List of all schedule rows.

```python
for row in schedule.rows:
    print(row.trip_id)
```

**Type:** `list[ScheduleRow]`

#### `revenue_trip_count`

Count of revenue (passenger-carrying) trips.

```python
print(f"Revenue trips: {schedule.revenue_trip_count}")
```

**Type:** `int`

### Methods

#### `__len__()`

Get total number of rows.

```python
print(f"Total rows: {len(schedule)}")
```

#### `block_ids()`

Get unique block identifiers.

```python
blocks = schedule.block_ids()
# ['B001', 'B002', 'B003']
```

**Returns:** `list[str]`

#### `run_numbers()`

Get unique run (driver) numbers.

```python
runs = schedule.run_numbers()
```

**Returns:** `list[str]`

#### `depots()`

Get unique depot codes.

```python
depots = schedule.depots()
```

**Returns:** `list[str]`

#### `trip_ids()`

Get unique trip identifiers.

```python
trips = schedule.trip_ids()
```

**Returns:** `list[str]`

#### `summary()`

Get summary statistics.

```python
stats = schedule.summary()
# {
#     'total_rows': 150,
#     'revenue_trips': 120,
#     'deadheads': 25,
#     'breaks_and_reliefs': 5,
#     'unique_blocks': 10,
#     'unique_runs': 8,
#     'unique_depots': 2
# }
```

**Returns:** `dict[str, int]`

#### `validate(gtfs, config=None)`

Validate the schedule against GTFS data.

```python
from transit_parser import GtfsFeed, ValidationConfig

gtfs = GtfsFeed.from_path("gtfs/")
config = ValidationConfig(gtfs_compliance="standard")
result = schedule.validate(gtfs, config)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `gtfs` | `GtfsFeed` | GTFS feed to validate against |
| `config` | `ValidationConfig \| None` | Validation configuration |

**Returns:** `ValidationResult`

#### `validate_structure(config=None)`

Validate schedule structure without GTFS reference checking.

```python
result = schedule.validate_structure()
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `config` | `ValidationConfig \| None` | Validation configuration |

**Returns:** `ValidationResult`

#### `infer_deadheads(gtfs=None, default_depot=None)`

Infer missing deadhead movements (pull-out, pull-in, interlining).

```python
result = schedule.infer_deadheads(gtfs, default_depot="DEPOT")
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `gtfs` | `GtfsFeed \| None` | GTFS feed for coordinate lookup |
| `default_depot` | `str \| None` | Default depot code for pull-out/pull-in |

**Returns:** `DeadheadInferenceResult`

#### `to_csv(path, columns=None, preset=None)`

Export schedule to CSV file.

```python
# With preset
schedule.to_csv("output.csv", preset="optibus")

# With custom columns
schedule.to_csv("output.csv", columns=["block", "trip_id", "start_time"])
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `str` | Output file path |
| `columns` | `list[str] \| None` | Custom column list |
| `preset` | `str \| None` | Preset name (see below) |

**Presets:**

| Name | Description |
|------|-------------|
| `default` | Standard columns |
| `minimal` | Essential columns only |
| `extended` | All available columns |
| `optibus` | Optibus-like format |
| `hastus` | Hastus-like format |
| `gtfs_block` | GTFS blocks.txt compatible |

#### `to_csv_string(columns=None, preset=None)`

Export schedule to CSV string.

```python
csv_str = schedule.to_csv_string(preset="minimal")
```

**Parameters:** Same as `to_csv()`

**Returns:** `str`

---

## ScheduleRow

```python
from transit_parser import ScheduleRow
```

A single row in a schedule file representing one movement or activity.

### Constructor

#### `ScheduleRow()`

Create an empty schedule row.

```python
row = ScheduleRow()
row.block = "B001"
row.trip_id = "T001"
row.start_time = "08:00:00"
```

### Properties

All properties are read/write unless noted.

| Property | Type | Description |
|----------|------|-------------|
| `run_number` | `str \| None` | Run (driver) identifier |
| `block` | `str \| None` | Block (vehicle) identifier |
| `start_place` | `str \| None` | Origin stop_id or location |
| `end_place` | `str \| None` | Destination stop_id or location |
| `start_time` | `str \| None` | Departure time (HH:MM:SS) |
| `end_time` | `str \| None` | Arrival time (HH:MM:SS) |
| `trip_id` | `str \| None` | GTFS trip_id (revenue only) |
| `depot` | `str \| None` | Depot code |
| `vehicle_class` | `str \| None` | Vehicle class (read-only) |
| `vehicle_type` | `str \| None` | Vehicle type (read-only) |
| `start_lat` | `float \| None` | Start latitude (read-only) |
| `start_lon` | `float \| None` | Start longitude (read-only) |
| `end_lat` | `float \| None` | End latitude (read-only) |
| `end_lon` | `float \| None` | End longitude (read-only) |
| `route_shape_id` | `str \| None` | GTFS shape_id (read-only) |
| `row_type` | `str` | Row type (read-only) |
| `duty_id` | `str \| None` | Duty identifier (read-only) |
| `shift_id` | `str \| None` | Shift identifier (read-only) |

**Row Types:**

| Value | Description |
|-------|-------------|
| `revenue` | Revenue trip with passengers |
| `pull_out` | Depot to first stop |
| `pull_in` | Last stop to depot |
| `deadhead` | Non-revenue between trips |
| `break` | Driver break |
| `relief` | Driver relief/changeover |
| `layover` | Layover at stop |

### Methods

#### `is_revenue()`

Check if this is a revenue (passenger-carrying) trip.

```python
if row.is_revenue():
    print(f"Trip: {row.trip_id}")
```

**Returns:** `bool`

#### `is_deadhead()`

Check if this is any type of deadhead movement.

```python
if row.is_deadhead():
    print(f"Deadhead: {row.start_place} → {row.end_place}")
```

**Returns:** `bool`

#### `duration_seconds()`

Get duration in seconds.

```python
duration = row.duration_seconds()
if duration:
    print(f"Duration: {duration // 60} minutes")
```

**Returns:** `int | None`

---

## ValidationConfig

```python
from transit_parser import ValidationConfig
```

Configuration for schedule validation.

### Constructor

#### `ValidationConfig(...)`

Create a validation configuration.

```python
config = ValidationConfig(
    gtfs_compliance="standard",
    min_layover_seconds=300,
    max_duty_length_seconds=32400,
)
```

**Parameters:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `gtfs_compliance` | `str \| None` | `"standard"` | Compliance level |
| `min_layover_seconds` | `int \| None` | `300` | Min time between trips (5 min) |
| `max_trip_duration_seconds` | `int \| None` | `14400` | Max trip length (4 hr) |
| `max_duty_length_seconds` | `int \| None` | `32400` | Max duty length (9 hr) |
| `max_continuous_driving_seconds` | `int \| None` | `16200` | Max driving before break (4.5 hr) |
| `min_break_duration_seconds` | `int \| None` | `1800` | Min break length (30 min) |
| `time_tolerance_seconds` | `int \| None` | `60` | Allowed GTFS time deviation |
| `validate_block_continuity` | `bool \| None` | `True` | Check block continuity |
| `validate_duty_constraints` | `bool \| None` | `True` | Check duty constraints |
| `generate_warnings` | `bool \| None` | `True` | Generate warning messages |

**GTFS Compliance Levels:**

| Level | Description |
|-------|-------------|
| `strict` | All trip_ids must exist in GTFS |
| `standard` | Some missing allowed with warnings |
| `lenient` | Skip GTFS reference checks |

### Class Methods

#### `ValidationConfig.strict()`

Create a strict validation config.

```python
config = ValidationConfig.strict()
```

**Returns:** `ValidationConfig`

#### `ValidationConfig.lenient()`

Create a lenient validation config.

```python
config = ValidationConfig.lenient()
```

**Returns:** `ValidationConfig`

---

## ValidationResult

```python
from transit_parser import ValidationResult
```

Result of schedule validation.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `is_valid` | `bool` | True if no errors |
| `error_count` | `int` | Number of errors |
| `warning_count` | `int` | Number of warnings |
| `errors` | `list[dict]` | Error details |
| `warnings` | `list[dict]` | Warning details |
| `rows_validated` | `int` | Rows checked |
| `blocks_validated` | `int` | Blocks checked |

### Error/Warning Format

Each error and warning is a dict with:

```python
{
    'code': 'E001',           # Error/warning code
    'category': 'GtfsIntegrity',  # Category
    'message': 'Trip T001 not found in GTFS',
    'context': 'row 5, block B001'  # Additional context
}
```

---

## DeadheadInferenceResult

```python
from transit_parser import DeadheadInferenceResult
```

Result of deadhead inference.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `pull_out_count` | `int` | Inferred pull-outs |
| `pull_in_count` | `int` | Inferred pull-ins |
| `interlining_count` | `int` | Inferred interlinings |
| `total_count` | `int` | Total inferred |
| `incomplete_blocks` | `list[str]` | Blocks that couldn't be completed |
