# Benchmark Suite

Performance benchmarks comparing `transit-parser` against common Python libraries.

## Quick Start

```bash
# Ensure release build (IMPORTANT - debug builds are 12x slower!)
uv run maturin develop --release

# Run unified benchmark suite and generate BENCH.md
uv run python benchmarks/run_benchmarks.py
```

**Latest results: [BENCH.md](BENCH.md)**

## What Gets Benchmarked

### GTFS Operations

| Benchmark | Description |
|-----------|-------------|
| **GTFS Loading** | Time to load a GTFS feed into memory |
| **stop_times Access** | Time to access stop_times data |
| **DataFrame Generation** | Time to get stop_times as pandas DataFrame |

### TXC Operations

| Benchmark | Description |
|-----------|-------------|
| **TXC Parsing** | Time to parse TransXChange XML |
| **TXC → GTFS Conversion** | Time to convert TXC to GTFS format |

## Libraries Compared

| Library | Type | Notes |
|---------|------|-------|
| **transit-parser** | Rust + Python | This library |
| partridge | Python | Lazy-loading GTFS with pandas |
| gtfs-kit | Python | GTFS analysis toolkit |
| lxml | Python/C | XML parsing baseline |
| transx2gtfs | Python | TXC converter (often fails) |

## Running Benchmarks

### Unified Runner (Recommended)

```bash
# Generates BENCH.md with ASCII charts and rankings
uv run python benchmarks/run_benchmarks.py
```

### Individual Benchmarks

```bash
# TXC parsing and conversion
uv run python benchmarks/bench_txc_parsing.py

# GTFS read/write with comparisons
uv run python benchmarks/bench_gtfs_parsing.py

# Lazy vs eager loading comparison
uv run python benchmarks/bench_lazy_gtfs.py
```

### pytest-benchmark (Detailed Statistics)

```bash
# Run with detailed statistics
uv run pytest benchmarks/ -v --benchmark-only

# Save and compare baselines
uv run pytest benchmarks/ --benchmark-save=baseline
uv run pytest benchmarks/ --benchmark-compare=baseline
```

### Native Rust Benchmark

```bash
# Test raw Rust performance (no Python overhead)
cargo run --release -p txc-parser --bin bench_txc -- path/to/file.xml
```

## Test Data

Benchmarks require:
- **TXC**: A TransXChange XML file (any UK bus operator schedule)
- **GTFS**: A GTFS feed directory or the converted output

Configure via environment variables:
```bash
export BENCH_TXC_FILE=/path/to/your/file.xml
export BENCH_GTFS_DIR=/path/to/your/gtfs/
uv run python benchmarks/run_benchmarks.py
```

Or pass paths directly in your benchmark scripts.

## Installing Comparison Libraries

```bash
uv pip install gtfs-kit partridge lxml pandas
```

Note: `pytxc` requires Python <3.12 due to shapely dependency issues.

## Performance Notes

### 1. Always Use Release Builds

```bash
uv run maturin develop --release  # ✅ Fast
uv run maturin develop            # ❌ 12x slower (debug)
```

### 2. Loading Strategies

| Strategy | Initial Load | Data Access | Use When |
|----------|-------------|-------------|----------|
| `GtfsFeed` (eager) | ~120ms | instant | Need all data |
| `LazyGtfsFeed` | ~0.01ms | ~130ms first | Selective access |
| `GtfsDataFrames` | ~0.01ms | ~300ms first | Need DataFrames |

### 3. Where transit-parser Excels

- **TXC parsing**: Only working high-performance option
- **TXC→GTFS conversion**: transx2gtfs has compatibility issues
- **Type safety**: Full IDE completion with typed objects
- **Flexibility**: Eager, lazy, or DataFrame output modes
