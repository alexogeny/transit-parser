# Transit Parser

**High-performance Python+Rust library for parsing GTFS and TXC transit data.**

[![Python 3.9+](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/downloads/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Features

- 🚀 **Blazing Fast** - Rust-powered parsing with Python convenience
- 📦 **Multiple Formats** - GTFS, TransXChange (TXC), CSV, JSON
- 🔄 **TXC → GTFS Conversion** - Convert UK bus data to standard GTFS
- 🔍 **Filtering API** - Query by route, stop, date, and more
- 📊 **Lazy Loading** - Defer parsing until you need the data
- 🐼 **DataFrame Support** - Optional pandas integration

## Quick Example

```python
from transit_parser import GtfsFeed, TxcDocument, TxcToGtfsConverter
from transit_parser.filtering import GtfsFilter

# Load a GTFS feed
feed = GtfsFeed.from_path("path/to/gtfs/")
print(f"Routes: {feed.route_count}, Trips: {feed.trip_count}")

# Filter by route
f = GtfsFilter(feed)
route_1_trips = f.trips_for_route("route_1")
stops = f.stops_for_route("route_1")

# Find active services on a date
active = f.active_services_on("2025-07-04")

# Convert TXC to GTFS
txc = TxcDocument.from_path("service.xml")
converter = TxcToGtfsConverter()
result = converter.convert(txc)
result.feed.to_zip("output.zip")
```

## Performance

Transit Parser is designed for speed. See the [Performance](performance.md) page for benchmarks.

| Operation | Transit Parser | partridge | gtfs-kit |
|-----------|---------------|-----------|----------|
| Load feed | 0.01ms (lazy) | 120ms | 850ms |
| stop_times | 136ms | 410ms | N/A |

## Installation

```bash
pip install transit-parser
```

Or with uv:

```bash
uv add transit-parser
```

## Next Steps

- [Quick Start Guide](getting-started/quickstart.md) - Get up and running in 5 minutes
- [API Reference](api/index.md) - Detailed API documentation
- [Filtering Guide](guide/filtering.md) - Learn to query your data
