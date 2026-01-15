# TXC to GTFS Conversion

Convert UK TransXChange data to the standard GTFS format.

## Basic Conversion

```python
from transit_parser import TxcDocument, TxcToGtfsConverter

# Load TXC document
txc = TxcDocument.from_path("service.xml")

# Create converter and convert
converter = TxcToGtfsConverter()
result = converter.convert(txc)

# Access the converted GTFS feed
feed = result.feed
print(f"Trips: {feed.trip_count}")

# Save to disk
feed.to_zip("output.zip")
```

## Conversion Options

Customize the conversion with `ConversionOptions`:

```python
from transit_parser import ConversionOptions, TxcToGtfsConverter

options = ConversionOptions(
    include_shapes=True,           # Generate shapes from route sections
    calendar_start="2025-01-01",   # Override calendar start date
    calendar_end="2025-12-31",     # Override calendar end date
    region="england",              # UK region for bank holidays
    default_timezone="Europe/London",
    default_agency_url="https://example.com",
)

converter = TxcToGtfsConverter(options)
result = converter.convert(txc)
```

### Options Reference

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `include_shapes` | bool | False | Generate shapes from route sections |
| `calendar_start` | str | None | Override start date (YYYY-MM-DD) |
| `calendar_end` | str | None | Override end date (YYYY-MM-DD) |
| `region` | str | "england" | UK region for bank holidays |
| `default_timezone` | str | "Europe/London" | Timezone if not specified |
| `default_agency_url` | str | "https://example.com" | Agency URL if not specified |

## Conversion Result

The `ConversionResult` provides:

```python
result = converter.convert(txc)

# The converted GTFS feed
feed = result.feed

# Conversion statistics
stats = result.stats
print(f"Agencies: {stats.agencies_converted}")
print(f"Routes: {stats.routes_converted}")
print(f"Trips: {stats.trips_converted}")
print(f"Stop times: {stats.stop_times_generated}")
print(f"Calendar entries: {stats.calendar_entries}")

# Any warnings during conversion
for warning in result.warnings:
    print(f"Warning: {warning}")
```

## Batch Conversion

Convert multiple TXC documents at once:

```python
from pathlib import Path

# Load multiple documents
docs = []
for path in Path("txc_files/").glob("*.xml"):
    docs.append(TxcDocument.from_path(str(path)))

# Batch convert
converter = TxcToGtfsConverter()
result = converter.convert_batch(docs)

# Single merged GTFS feed
feed = result.feed
feed.to_zip("merged_output.zip")
```

## Mapping Reference

| TXC Element | GTFS File | Notes |
|-------------|-----------|-------|
| Operator | agency.txt | Direct mapping |
| Service/Line | routes.txt | Line → route |
| StopPoints | stops.txt | Uses NaPTAN codes |
| VehicleJourney | trips.txt | Each VJ = one trip |
| Timing patterns | stop_times.txt | Calculated from patterns |
| OperatingPeriod | calendar.txt | Date range |
| RegularDayType | calendar.txt | Day flags |
| BankHolidayOperation | calendar_dates.txt | Exception handling |
| RouteSection | shapes.txt | If include_shapes=True |

## UK Bank Holidays

The converter handles UK bank holidays based on the region:

- **england** - English bank holidays
- **scotland** - Scottish bank holidays (includes St Andrew's Day)
- **wales** - Welsh bank holidays
- **northern_ireland** - NI bank holidays (includes St Patrick's Day)

Bank holiday exceptions are added to `calendar_dates.txt` based on the TXC `BankHolidayOperation` elements.
