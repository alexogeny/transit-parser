# Filtering API Reference

## GtfsFilter

::: transit_parser.filtering.GtfsFilter
    options:
      show_source: false
      heading_level: 3

### Constructor

```python
GtfsFilter(feed: GtfsFeed | LazyGtfsFeed)
```

Create a filter for the given GTFS feed.

### Lookup Methods

#### get_stop
```python
f.get_stop(stop_id: str) -> Stop | None
```
Get a stop by its ID.

#### get_route
```python
f.get_route(route_id: str) -> Route | None
```
Get a route by its ID.

#### get_trip
```python
f.get_trip(trip_id: str) -> Trip | None
```
Get a trip by its ID.

#### get_agency
```python
f.get_agency(agency_id: str) -> Agency | None
```
Get an agency by its ID.

#### get_calendar
```python
f.get_calendar(service_id: str) -> Calendar | None
```
Get a calendar entry by service ID.

### Route Filtering

#### trips_for_route
```python
f.trips_for_route(route_id: str) -> List[Trip]
```
Get all trips for a specific route.

#### stop_times_for_route
```python
f.stop_times_for_route(route_id: str) -> List[StopTime]
```
Get all stop times for trips on a route.

#### stops_for_route
```python
f.stops_for_route(route_id: str) -> List[Stop]
```
Get all unique stops served by a route.

### Trip Filtering

#### stop_times_for_trip
```python
f.stop_times_for_trip(trip_id: str) -> List[StopTime]
```
Get all stop times for a trip, sorted by stop_sequence.

#### stops_for_trip
```python
f.stops_for_trip(trip_id: str) -> List[Stop]
```
Get all stops for a trip, in sequence order.

### Stop Filtering

#### stop_times_at_stop
```python
f.stop_times_at_stop(stop_id: str) -> List[StopTime]
```
Get all stop times at a specific stop.

#### trips_serving_stop
```python
f.trips_serving_stop(stop_id: str) -> List[Trip]
```
Get all trips that serve a stop.

#### routes_serving_stop
```python
f.routes_serving_stop(stop_id: str) -> List[Route]
```
Get all routes that serve a stop.

### Agency Filtering

#### routes_for_agency
```python
f.routes_for_agency(agency_id: str) -> List[Route]
```
Get all routes operated by an agency.

#### trips_for_agency
```python
f.trips_for_agency(agency_id: str) -> List[Trip]
```
Get all trips for routes operated by an agency.

### Service/Date Filtering

#### trips_for_service
```python
f.trips_for_service(service_id: str) -> List[Trip]
```
Get all trips for a specific service.

#### active_services_on
```python
f.active_services_on(date_input: str | date) -> List[Calendar]
```
Get all services active on a specific date.

Takes into account both calendar (day-of-week patterns) and calendar_dates (exceptions).

**Parameters:**
- `date_input`: Date as string ("YYYY-MM-DD" or "YYYYMMDD") or `datetime.date` object

**Raises:**
- `InvalidDateError`: If the date string cannot be parsed

#### trips_on_date
```python
f.trips_on_date(date_input: str | date) -> List[Trip]
```
Get all trips running on a specific date.

### Shape Queries

#### shape_for_trip
```python
f.shape_for_trip(trip_id: str) -> Shape | None
```
Get the shape for a trip, if one is assigned.

### Statistics

#### route_stop_count
```python
f.route_stop_count(route_id: str) -> int
```
Get the number of unique stops served by a route.

#### route_trip_count
```python
f.route_trip_count(route_id: str) -> int
```
Get the number of trips for a route.

#### stop_trip_count
```python
f.stop_trip_count(stop_id: str) -> int
```
Get the number of trips serving a stop.
