# Loading GTFS Feeds

This guide covers all the ways to load and work with GTFS data.

## GtfsFeed vs LazyGtfsFeed

Transit Parser provides two ways to load GTFS feeds:

| Feature | GtfsFeed | LazyGtfsFeed |
|---------|----------|--------------|
| Load time | Parses everything | Instant |
| Memory | All data in memory | On-demand |
| Best for | Small-medium feeds | Large feeds |

### GtfsFeed (Eager Loading)

```python
from transit_parser import GtfsFeed

# All files are parsed immediately
feed = GtfsFeed.from_path("/path/to/gtfs/")
```

### LazyGtfsFeed (Lazy Loading)

```python
from transit_parser import LazyGtfsFeed

# No parsing happens yet
feed = LazyGtfsFeed.from_path("/path/to/gtfs/")

# Counts are fast (reads file metadata)
print(feed.stop_time_count)

# Data is parsed on first access
stop_times = feed.stop_times
```

## Loading Methods

### From Directory

```python
feed = GtfsFeed.from_path("/path/to/gtfs/")
```

The directory should contain the GTFS CSV files:
- `agency.txt` (required)
- `stops.txt` (required)
- `routes.txt` (required)
- `trips.txt` (required)
- `stop_times.txt` (required)
- `calendar.txt` (conditionally required)
- `calendar_dates.txt` (conditionally required)
- `shapes.txt` (optional)

### From ZIP File

```python
feed = GtfsFeed.from_zip("/path/to/gtfs.zip")
```

### From Bytes

```python
with open("/path/to/gtfs.zip", "rb") as f:
    data = f.read()

feed = GtfsFeed.from_bytes(data)
```

## Accessing Data

### Agencies

```python
for agency in feed.agencies:
    print(f"ID: {agency.id}")
    print(f"Name: {agency.name}")
    print(f"URL: {agency.url}")
    print(f"Timezone: {agency.timezone}")
```

### Stops

```python
for stop in feed.stops:
    print(f"ID: {stop.id}")
    print(f"Name: {stop.name}")
    print(f"Code: {stop.code}")
    print(f"Coordinates: ({stop.latitude}, {stop.longitude})")
```

### Routes

```python
for route in feed.routes:
    print(f"ID: {route.id}")
    print(f"Short name: {route.short_name}")
    print(f"Long name: {route.long_name}")
    print(f"Type: {route.route_type}")  # 0=tram, 1=subway, 2=rail, 3=bus...
```

### Trips

```python
for trip in feed.trips:
    print(f"ID: {trip.id}")
    print(f"Route: {trip.route_id}")
    print(f"Service: {trip.service_id}")
    print(f"Headsign: {trip.headsign}")
```

### Stop Times

```python
for st in feed.stop_times:
    print(f"Trip: {st.trip_id}")
    print(f"Stop: {st.stop_id}")
    print(f"Sequence: {st.stop_sequence}")
    print(f"Arrival: {st.arrival_time}")
    print(f"Departure: {st.departure_time}")
```

### Calendar

```python
for cal in feed.calendars:
    print(f"Service: {cal.service_id}")
    print(f"Mon-Fri: {cal.monday}-{cal.friday}")
    print(f"Sat-Sun: {cal.saturday}-{cal.sunday}")
    print(f"Dates: {cal.start_date} to {cal.end_date}")
```

### Calendar Dates

```python
for cd in feed.calendar_dates:
    print(f"Service: {cd.service_id}")
    print(f"Date: {cd.date}")
    print(f"Exception: {cd.exception_type}")  # 1=added, 2=removed
```

### Shapes

```python
for shape in feed.shapes:
    print(f"ID: {shape.id}")
    print(f"Points: {len(shape.points)}")
    for lat, lon, seq in shape.points:
        print(f"  {seq}: ({lat}, {lon})")
```

## Writing GTFS

### To Directory

```python
feed.to_path("/path/to/output/")
```

### To ZIP File

```python
feed.to_zip("/path/to/output.zip")
```

### To Bytes

```python
data = feed.to_bytes()
```

## Count Properties

Get entity counts without loading all data:

```python
feed.agency_count
feed.stop_count
feed.route_count
feed.trip_count
feed.stop_time_count
feed.calendar_count
feed.calendar_date_count
feed.shape_count
```

## Converting Lazy to Eager

```python
lazy = LazyGtfsFeed.from_path("/path/to/gtfs/")

# Convert to regular GtfsFeed
eager = lazy.materialize()
```
