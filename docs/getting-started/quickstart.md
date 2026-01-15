# Quick Start

This guide will get you up and running with transit-parser in under 5 minutes.

## Loading a GTFS Feed

The most common operation is loading a GTFS feed:

```python
from transit_parser import GtfsFeed

# From a directory
feed = GtfsFeed.from_path("/path/to/gtfs/")

# From a ZIP file
feed = GtfsFeed.from_zip("/path/to/gtfs.zip")

# Check what's in the feed
print(f"Agencies: {feed.agency_count}")
print(f"Stops: {feed.stop_count}")
print(f"Routes: {feed.route_count}")
print(f"Trips: {feed.trip_count}")
print(f"Stop times: {feed.stop_time_count}")
```

## Accessing Data

All GTFS entities are available as Python objects:

```python
# Iterate over routes
for route in feed.routes:
    print(f"{route.id}: {route.short_name} - {route.long_name}")

# Get all stops
for stop in feed.stops:
    print(f"{stop.name} ({stop.latitude}, {stop.longitude})")

# Access trip details
trip = feed.trips[0]
print(f"Trip {trip.id} on route {trip.route_id}")
```

## Lazy Loading for Large Feeds

For large feeds, use `LazyGtfsFeed` to defer parsing:

```python
from transit_parser import LazyGtfsFeed

# This is instant - no parsing yet
feed = LazyGtfsFeed.from_path("/path/to/large/gtfs/")

# Counts are available immediately
print(f"Stop times: {feed.stop_time_count}")  # Fast!

# Data is parsed on first access
stop_times = feed.stop_times  # Parses stop_times.txt now
```

## Filtering Data

Use the filtering API for queries:

```python
from transit_parser import GtfsFeed
from transit_parser.filtering import GtfsFilter

feed = GtfsFeed.from_path("/path/to/gtfs/")
f = GtfsFilter(feed)

# Get trips for a route
trips = f.trips_for_route("route_1")

# Get stops served by a route
stops = f.stops_for_route("route_1")

# Find active services on a date
services = f.active_services_on("2025-07-04")

# Get trips running on a specific date
trips = f.trips_on_date("2025-07-04")
```

## Converting TXC to GTFS

Convert UK TransXChange data to GTFS:

```python
from transit_parser import TxcDocument, TxcToGtfsConverter

# Load TXC document
txc = TxcDocument.from_path("service.xml")
print(f"Service: {txc.get_service_codes()}")
print(f"Stops: {txc.stop_point_count}")

# Convert to GTFS
converter = TxcToGtfsConverter()
result = converter.convert(txc)

# Access the converted feed
feed = result.feed
print(f"Converted {feed.trip_count} trips")

# Save to disk
feed.to_zip("output.zip")
```

## Error Handling

Use the built-in exceptions for robust error handling:

```python
from transit_parser import GtfsFeed, GtfsFileNotFoundError, InvalidDateError
from transit_parser.filtering import GtfsFilter

try:
    feed = GtfsFeed.from_path("/nonexistent/path")
except GtfsFileNotFoundError as e:
    print(f"Feed not found: {e.path}")

try:
    f = GtfsFilter(feed)
    f.active_services_on("invalid-date")
except InvalidDateError as e:
    print(f"Bad date: {e.date_string}")
```

## Next Steps

- [GTFS Guide](../guide/gtfs.md) - Deep dive into GTFS loading
- [Filtering Guide](../guide/filtering.md) - Advanced filtering techniques
- [API Reference](../api/index.md) - Complete API documentation
