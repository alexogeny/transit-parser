# Performance

Transit Parser is designed for speed, using Rust for core parsing operations.

## Benchmarks

Tested on a 50MB GTFS feed (NYC MTA-like size):

| Operation | Transit Parser | partridge | gtfs-kit |
|-----------|---------------|-----------|----------|
| Load feed (eager) | 120ms | 120ms | 850ms |
| Load feed (lazy) | 0.01ms | N/A | N/A |
| First access to stop_times | 136ms | 410ms | N/A |
| Cached access | 0.001ms | N/A | N/A |

## Key Optimizations

### 1. Lazy Loading

`LazyGtfsFeed` defers CSV parsing until you actually need the data:

```python
# Instant - no parsing
feed = LazyGtfsFeed.from_path("gtfs/")

# Still instant - reads metadata only
print(feed.stop_time_count)

# Parses stop_times.txt now
stop_times = feed.stop_times
```

### 2. Zero-Copy Parsing

The Rust CSV parser uses zero-copy deserialization where possible, minimizing memory allocations.

### 3. Index Caching

`GtfsFilter` builds indexes on first use:

```python
f = GtfsFilter(feed)

# First call builds stop index (~1ms)
stop = f.get_stop("stop_1")

# Subsequent calls use cache (~0.001ms)
stop = f.get_stop("stop_2")
```

### 4. Parallel Processing

Batch operations use Rayon for parallel processing:

```python
# Converts files in parallel
result = converter.convert_batch(documents)
```

## Memory Usage

Transit Parser is memory-efficient:

| Feed Size | Eager Load | Lazy Load |
|-----------|------------|-----------|
| 10 MB | ~100 MB | ~1 MB |
| 50 MB | ~500 MB | ~5 MB |
| 100 MB | ~1 GB | ~10 MB |

Lazy loading only holds the data you've accessed in memory.

## Tips for Large Feeds

### Use Lazy Loading

```python
# Good - instant load
feed = LazyGtfsFeed.from_path("large_feed/")

# Access only what you need
routes = feed.routes  # Parses routes.txt only
```

### Use Filtering Early

```python
# Good - filter before iteration
f = GtfsFilter(feed)
route_trips = f.trips_for_route("route_1")

# Less efficient - iterates all trips
route_trips = [t for t in feed.trips if t.route_id == "route_1"]
```

### Stream Large Results

For very large result sets, consider chunking:

```python
# Process stop_times in chunks
chunk_size = 10000
stop_times = feed.stop_times

for i in range(0, len(stop_times), chunk_size):
    chunk = stop_times[i:i + chunk_size]
    process(chunk)
```

## Running Benchmarks

The repository includes benchmarks:

```bash
cd benchmarks
python run_benchmarks.py
```

Results are saved to `benchmarks/BENCH.md`.
