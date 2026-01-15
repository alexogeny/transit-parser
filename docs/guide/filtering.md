# Filtering Data

The `GtfsFilter` class provides powerful querying capabilities for GTFS data.

## Getting Started

```python
from transit_parser import GtfsFeed
from transit_parser.filtering import GtfsFilter

feed = GtfsFeed.from_path("/path/to/gtfs/")
f = GtfsFilter(feed)
```

The filter works with both `GtfsFeed` and `LazyGtfsFeed`.

## Lookup by ID

Get individual entities by their ID:

```python
# Get a specific stop
stop = f.get_stop("stop_123")
if stop:
    print(f"{stop.name} at ({stop.latitude}, {stop.longitude})")

# Get a specific route
route = f.get_route("route_1")

# Get a specific trip
trip = f.get_trip("trip_abc")

# Get a specific agency
agency = f.get_agency("agency_1")

# Get calendar for a service
calendar = f.get_calendar("weekday")
```

## Filter by Route

```python
# Get all trips for a route
trips = f.trips_for_route("route_1")

# Get all stop times for a route
stop_times = f.stop_times_for_route("route_1")

# Get all unique stops served by a route
stops = f.stops_for_route("route_1")
```

## Filter by Trip

```python
# Get stop times for a trip (sorted by sequence)
stop_times = f.stop_times_for_trip("trip_1")

# Get stops for a trip (in order)
stops = f.stops_for_trip("trip_1")
```

## Filter by Stop

```python
# Get all stop times at a stop
stop_times = f.stop_times_at_stop("stop_123")

# Get all trips that serve a stop
trips = f.trips_serving_stop("stop_123")

# Get all routes that serve a stop
routes = f.routes_serving_stop("stop_123")
```

## Filter by Agency

```python
# Get all routes for an agency
routes = f.routes_for_agency("agency_1")

# Get all trips for an agency
trips = f.trips_for_agency("agency_1")
```

## Filter by Date

Find what services run on a specific date:

```python
from datetime import date

# Get active services on a date (handles calendar + calendar_dates)
services = f.active_services_on("2025-07-04")
# or
services = f.active_services_on(date(2025, 7, 4))

for svc in services:
    print(f"Service {svc.service_id} is running")

# Get all trips running on a date
trips = f.trips_on_date("2025-07-04")
print(f"{len(trips)} trips running on July 4th")
```

The date filtering correctly handles:
- Regular calendar (day-of-week patterns)
- Calendar dates (exceptions - added/removed services)
- Date ranges (start_date/end_date)

## Filter by Service

```python
# Get all trips for a specific service
weekday_trips = f.trips_for_service("weekday")
```

## Shape Queries

```python
# Get the shape for a trip
shape = f.shape_for_trip("trip_1")
if shape:
    print(f"Shape {shape.id} has {len(shape.points)} points")
```

## Statistics

Quick counts without loading full data:

```python
# Number of unique stops on a route
stop_count = f.route_stop_count("route_1")

# Number of trips on a route
trip_count = f.route_trip_count("route_1")

# Number of trips serving a stop
trip_count = f.stop_trip_count("stop_123")
```

## Index Caching

The filter builds indexes on first use for fast lookups:

```python
# First call builds the stop index
stop1 = f.get_stop("stop_1")  # ~1ms

# Subsequent calls use the cached index
stop2 = f.get_stop("stop_2")  # ~0.001ms
```

## Error Handling

Invalid dates raise `InvalidDateError`:

```python
from transit_parser import InvalidDateError

try:
    f.active_services_on("not-a-date")
except InvalidDateError as e:
    print(f"Invalid date: {e.date_string}")
    print(f"Expected format: {e.expected_format}")
```

## Example: Building a Departure Board

```python
from datetime import date

feed = GtfsFeed.from_path("/path/to/gtfs/")
f = GtfsFilter(feed)

# Get stop times at a station
stop_id = "central_station"
today = date.today()

# Get trips running today
active_services = {s.service_id for s in f.active_services_on(today)}

# Get departures from this stop
stop_times = f.stop_times_at_stop(stop_id)

# Filter to today's services
departures = []
for st in stop_times:
    trip = f.get_trip(st.trip_id)
    if trip and trip.service_id in active_services:
        route = f.get_route(trip.route_id)
        departures.append({
            "time": st.departure_time,
            "route": route.short_name if route else "?",
            "headsign": trip.headsign,
        })

# Sort by departure time
departures.sort(key=lambda x: x["time"])

for d in departures[:10]:
    print(f"{d['time']} - {d['route']} - {d['headsign']}")
```
