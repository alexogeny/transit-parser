# Transit Parser

High-performance Python+Rust library for parsing transit data formats with TXC to GTFS conversion.

## Features

- **GTFS Static** - Parse and write GTFS feeds (CSV-based)
- **TransXChange (TXC)** - Parse UK XML transit format
- **TXC to GTFS** - Convert TransXChange to GTFS
- **Generic CSV/JSON** - Parse any CSV/JSON with schema inference

## Installation

### Prerequisites

- Python 3.9+
- Rust 1.75+ (with cargo)
- uv (recommended) or pip

### Development Setup

```bash
# Install uv if not already installed
curl -LsSf https://astral.sh/uv/install.sh | sh

# Clone and enter directory
cd parser

# Create virtual environment and install in dev mode
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Build and install with maturin
uv pip install maturin
maturin develop

# Or use pip directly
pip install maturin
maturin develop
```

### Building for Release

```bash
maturin build --release
```

## Usage

### Parse GTFS Feed

```python
from transit_parser import GtfsFeed

# From ZIP file
feed = GtfsFeed.from_zip("path/to/gtfs.zip")

# From directory
feed = GtfsFeed.from_path("path/to/gtfs/")

# Access data
print(f"Agencies: {len(feed.agencies)}")
print(f"Routes: {len(feed.routes)}")
print(f"Stops: {len(feed.stops)}")
print(f"Trips: {len(feed.trips)}")

# Write to ZIP
feed.to_zip("output.zip")
```

### Parse TransXChange

```python
from transit_parser import TxcDocument

# From file
doc = TxcDocument.from_path("path/to/file.xml")

# From string
doc = TxcDocument.from_string(xml_string)

# Inspect document
print(f"Schema version: {doc.schema_version}")
print(f"Operators: {doc.operator_count}")
print(f"Services: {doc.service_count}")
print(f"Vehicle journeys: {doc.vehicle_journey_count}")
```

### Convert TXC to GTFS

```python
from transit_parser import TxcDocument, TxcToGtfsConverter, ConversionOptions

# Parse TXC
doc = TxcDocument.from_path("input.xml")

# Configure conversion
options = ConversionOptions(
    include_shapes=True,
    region="england",  # For bank holiday handling
    calendar_start="2024-01-01",
    calendar_end="2024-12-31",
)

# Convert
converter = TxcToGtfsConverter(options)
result = converter.convert(doc)

# Check results
print(f"Converted {result.stats.trips_converted} trips")
print(f"Warnings: {len(result.warnings)}")

# Save GTFS
result.feed.to_zip("output.zip")
```

### Batch Conversion

```python
from pathlib import Path
from transit_parser import TxcDocument, TxcToGtfsConverter

# Parse multiple TXC files
docs = []
for xml_file in Path("txc_files/").glob("*.xml"):
    docs.append(TxcDocument.from_path(str(xml_file)))

# Convert all to single GTFS
converter = TxcToGtfsConverter()
result = converter.convert_batch(docs)
result.feed.to_zip("combined.zip")
```

### Generic CSV Parsing

```python
from transit_parser import CsvDocument

# Parse with automatic type inference
doc = CsvDocument.from_path("data.csv")

print(f"Columns: {doc.columns}")
print(f"Rows: {len(doc)}")

# Access rows as dicts
for row in doc.rows:
    print(row)
```

### JSON Parsing

```python
from transit_parser import JsonDocument

# Parse JSON
doc = JsonDocument.from_path("data.json")

# Access root value
data = doc.root

# Use JSON pointer for nested access
value = doc.pointer("/data/items/0/name")
```

## Project Structure

```
parser/
├── pyproject.toml          # Python project config (maturin backend)
├── Cargo.toml              # Rust workspace root
├── rust/
│   ├── transit-core/       # Core data models and traits
│   ├── gtfs-parser/        # GTFS Static parser
│   ├── txc-parser/         # TransXChange parser
│   ├── txc-gtfs-adapter/   # TXC→GTFS conversion
│   ├── csv-parser/         # Generic CSV parser
│   ├── json-parser/        # Generic JSON parser
│   └── transit-bindings/   # PyO3 Python bindings
└── python/
    └── transit_parser/     # Python package
```

## Performance

The Rust core provides high performance for:

- **Streaming XML parsing** - Process large TXC files without loading entire DOM
- **Zero-copy CSV parsing** - Efficient GTFS file reading
- **Parallel processing** - Batch conversion uses multiple cores
- **GIL release** - Python can do other work during long operations

## License

MIT OR Apache-2.0
