# Schedule Validation & Generation

Transit Parser includes a powerful schedule validation and generation module for working with operational transit schedules. This guide covers how to validate schedules against GTFS data, infer missing deadheads, and export schedules in various formats.

## What is a Schedule?

A **schedule** (also called a "run cut" or "blocking") is an operational plan that assigns:

- **Trips** to **blocks** (vehicles)
- **Blocks** to **runs** (drivers)
- Times and locations for all movements

Schedules typically come from scheduling software like Optibus, Hastus, or GIRO and contain both:

- **Revenue trips** - Passenger-carrying trips that reference GTFS trip_ids
- **Non-revenue movements** - Deadheads (pull-out, pull-in, interlining), breaks, reliefs

## Loading a Schedule

### From CSV File

```python
from transit_parser import Schedule

# Load with automatic column detection
schedule = Schedule.from_csv("schedule.csv")

print(f"Rows: {len(schedule)}")
print(f"Blocks: {schedule.block_ids()}")
print(f"Trips: {schedule.trip_ids()}")
```

### From CSV String

```python
csv_data = """block,trip_id,start_time,end_time,start_place,end_place
B001,T001,08:00:00,08:30:00,STOP_A,STOP_B
B001,T002,08:35:00,09:05:00,STOP_B,STOP_C
B002,T003,08:15:00,08:45:00,STOP_C,STOP_D"""

schedule = Schedule.from_csv_string(csv_data)
```

### With Custom Column Mapping

If your CSV uses non-standard column names, provide a mapping:

```python
schedule = Schedule.from_csv_with_mapping(
    "schedule.csv",
    column_mapping={
        "block": "vehicle_id",
        "trip_id": "journey_ref",
        "start_time": "depart",
        "end_time": "arrive",
        "start_place": "origin",
        "end_place": "destination",
    }
)
```

## Schedule Summary

Get a quick overview of your schedule:

```python
summary = schedule.summary()
print(summary)
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

## Validating Schedules

### Against GTFS Data

The most powerful validation checks that schedule references match GTFS entities:

```python
from transit_parser import GtfsFeed, Schedule, ValidationConfig

# Load both
gtfs = GtfsFeed.from_path("gtfs/")
schedule = Schedule.from_csv("schedule.csv")

# Validate with default settings
result = schedule.validate(gtfs)

if result.is_valid:
    print("Schedule is valid!")
else:
    print(f"Found {result.error_count} errors:")
    for error in result.errors:
        print(f"  [{error['code']}] {error['message']}")
```

### Validation Levels

Configure how strict validation should be:

```python
# Strict - All trip_ids must exist in GTFS
config = ValidationConfig.strict()

# Standard - Some missing references allowed with warnings
config = ValidationConfig(gtfs_compliance="standard")

# Lenient - Only check structure, skip GTFS references
config = ValidationConfig.lenient()

result = schedule.validate(gtfs, config)
```

### Custom Business Rules

Configure business rules for your operation:

```python
config = ValidationConfig(
    gtfs_compliance="standard",

    # Minimum layover between trips (default: 5 minutes)
    min_layover_seconds=300,

    # Maximum single trip duration (default: 4 hours)
    max_trip_duration_seconds=14400,

    # Maximum duty length (default: 9 hours)
    max_duty_length_seconds=32400,

    # Maximum continuous driving before required break (default: 4.5 hours)
    max_continuous_driving_seconds=16200,

    # Minimum break duration (default: 30 minutes)
    min_break_duration_seconds=1800,

    # Allowed deviation from GTFS times (default: 60 seconds)
    time_tolerance_seconds=60,
)

result = schedule.validate(gtfs, config)
```

### Structure-Only Validation

Validate schedule structure without GTFS:

```python
# Check block continuity, time ordering, etc.
result = schedule.validate_structure()

for warning in result.warnings:
    print(f"Warning: {warning['message']}")
```

### Validation Result

The result contains detailed error and warning information:

```python
result = schedule.validate(gtfs, config)

print(f"Valid: {result.is_valid}")
print(f"Errors: {result.error_count}")
print(f"Warnings: {result.warning_count}")
print(f"Rows validated: {result.rows_validated}")
print(f"Blocks validated: {result.blocks_validated}")

# Errors are critical issues
for error in result.errors:
    print(f"ERROR [{error['code']}] {error['category']}: {error['message']}")
    if error['context']:
        print(f"  Context: {error['context']}")

# Warnings are best-practice suggestions
for warning in result.warnings:
    print(f"WARN [{warning['code']}]: {warning['message']}")
```

## Inferring Deadheads

Many schedules only contain revenue trips. The deadhead inferrer can generate missing:

- **Pull-outs** - From depot to first trip start
- **Pull-ins** - From last trip end to depot
- **Interlinings** - Between non-continuous trips in a block

```python
# Infer deadheads
result = schedule.infer_deadheads(
    gtfs=gtfs,  # Optional: for stop coordinates
    default_depot="MAIN_DEPOT"  # Depot code for pull-out/pull-in
)

print(f"Inferred {result.total_count} deadheads:")
print(f"  Pull-outs: {result.pull_out_count}")
print(f"  Pull-ins: {result.pull_in_count}")
print(f"  Interlinings: {result.interlining_count}")

# Check for blocks that couldn't be completed
if result.incomplete_blocks:
    print(f"Could not infer for blocks: {result.incomplete_blocks}")
```

## Exporting Schedules

### To CSV File

```python
# Export with default columns
schedule.to_csv("output.csv")

# Export specific columns
schedule.to_csv("output.csv", columns=[
    "run_number", "block", "trip_id", "start_time", "end_time"
])
```

### Using Presets

Presets provide column configurations that approximate common formats:

```python
# Optibus-like format
schedule.to_csv("optibus_schedule.csv", preset="optibus")

# Hastus-like format
schedule.to_csv("hastus_schedule.csv", preset="hastus")

# Minimal (just essential columns)
schedule.to_csv("minimal.csv", preset="minimal")

# Extended (all available columns)
schedule.to_csv("full.csv", preset="extended")

# GTFS blocks.txt compatible
schedule.to_csv("blocks.csv", preset="gtfs_block")
```

### To String

```python
csv_string = schedule.to_csv_string(preset="minimal")
print(csv_string)
```

## Working with Schedule Rows

Access individual rows for detailed analysis:

```python
for row in schedule.rows:
    print(f"Block {row.block}: {row.start_time} - {row.end_time}")

    if row.is_revenue():
        print(f"  Revenue trip: {row.trip_id}")
    elif row.is_deadhead():
        print(f"  Deadhead: {row.start_place} → {row.end_place}")

    # Duration in seconds
    if row.duration_seconds():
        minutes = row.duration_seconds() // 60
        print(f"  Duration: {minutes} minutes")
```

### Row Properties

Each `ScheduleRow` provides:

| Property | Type | Description |
|----------|------|-------------|
| `block` | `str \| None` | Block (vehicle) identifier |
| `run_number` | `str \| None` | Run (driver) identifier |
| `trip_id` | `str \| None` | GTFS trip_id (revenue trips only) |
| `start_place` | `str \| None` | Origin stop_id or location |
| `end_place` | `str \| None` | Destination stop_id or location |
| `start_time` | `str \| None` | Departure time (HH:MM:SS) |
| `end_time` | `str \| None` | Arrival time (HH:MM:SS) |
| `depot` | `str \| None` | Depot code |
| `vehicle_class` | `str \| None` | Vehicle class/category |
| `vehicle_type` | `str \| None` | Specific vehicle type |
| `row_type` | `str` | Type: revenue, pull_out, pull_in, deadhead, break, relief |
| `duty_id` | `str \| None` | Duty identifier (rostering) |
| `shift_id` | `str \| None` | Shift identifier (rostering) |

### Row Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `is_revenue()` | `bool` | True if this is a revenue trip |
| `is_deadhead()` | `bool` | True if this is any deadhead movement |
| `duration_seconds()` | `int \| None` | Duration in seconds |

## CSV Column Support

The schedule reader auto-detects these column names (case-insensitive):

| Field | Accepted Column Names |
|-------|----------------------|
| block | block, block_id, vehicle_block, veh_block |
| run_number | run, run_number, run_id, driver_run |
| trip_id | trip, trip_id, journey_id, journey_ref |
| start_place | start_place, origin, from, start_stop, from_stop |
| end_place | end_place, destination, to, end_stop, to_stop |
| start_time | start_time, depart, departure, start |
| end_time | end_time, arrive, arrival, end |
| depot | depot, garage, depot_code, garage_code |
| vehicle_class | vehicle_class, veh_class, class |
| vehicle_type | vehicle_type, veh_type, type |
| row_type | row_type, type, activity_type |

## Example: Complete Workflow

```python
from transit_parser import GtfsFeed, Schedule, ValidationConfig

# 1. Load data
gtfs = GtfsFeed.from_path("gtfs/")
schedule = Schedule.from_csv("raw_schedule.csv")

# 2. Get overview
print(f"Schedule has {len(schedule)} rows")
print(f"Summary: {schedule.summary()}")

# 3. Validate against GTFS
config = ValidationConfig(
    gtfs_compliance="standard",
    min_layover_seconds=180,  # 3 minute minimum layover
)
result = schedule.validate(gtfs, config)

if not result.is_valid:
    print("Validation failed!")
    for error in result.errors:
        print(f"  {error['message']}")
    # Handle errors...

# 4. Infer missing deadheads
inference = schedule.infer_deadheads(gtfs, default_depot="CENTRAL")
print(f"Added {inference.total_count} deadheads")

# 5. Export complete schedule
schedule.to_csv("complete_schedule.csv", preset="extended")
print("Done!")
```
