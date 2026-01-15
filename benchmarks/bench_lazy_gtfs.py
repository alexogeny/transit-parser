"""Benchmark comparing lazy vs eager GTFS loading."""

import os
import time
from pathlib import Path

# Our library
from transit_parser import GtfsFeed, LazyGtfsFeed

# Test directories - configurable via environment variables
GTFS_DIR = Path(os.environ.get("BENCH_GTFS_DIR", Path(__file__).parent.parent / "gtfs_output"))
TXC_FILE = Path(os.environ.get("BENCH_TXC_FILE", Path(__file__).parent.parent / "sample.xml"))

def check_dir_exists():
    """Check if test directory exists."""
    if not GTFS_DIR.exists():
        print(f"Test directory not found: {GTFS_DIR}")
        print("Creating test GTFS from TXC conversion...")
        from transit_parser import TxcDocument, TxcToGtfsConverter, ConversionOptions

        if not TXC_FILE.exists():
            print(f"TXC file not found: {TXC_FILE}")
            return False

        doc = TxcDocument.from_path(str(TXC_FILE))
        options = ConversionOptions(include_shapes=False, region="england")
        converter = TxcToGtfsConverter(options)
        result = converter.convert(doc)

        GTFS_DIR.mkdir(exist_ok=True)
        result.feed.to_path(str(GTFS_DIR))
        print(f"Created test GTFS at {GTFS_DIR}")
    return True


def run_benchmark():
    """Run benchmark comparing lazy vs eager loading."""
    if not check_dir_exists():
        return

    iterations = 5

    print(f"Benchmarking GTFS loading from: {GTFS_DIR}")
    print()

    # ============================================
    # Eager loading (GtfsFeed)
    # ============================================
    print("=" * 60)
    print("GtfsFeed (eager, parses all on load)")
    print("=" * 60)

    # Warmup
    _ = GtfsFeed.from_path(str(GTFS_DIR))

    load_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        feed = GtfsFeed.from_path(str(GTFS_DIR))
        load_times.append(time.perf_counter() - start)

    print(f"Load time (parses all CSVs):")
    print(f"  Mean: {sum(load_times) / len(load_times) * 1000:.2f} ms")
    print(f"  Min:  {min(load_times) * 1000:.2f} ms")

    # Access cached data
    access_times = []
    feed = GtfsFeed.from_path(str(GTFS_DIR))
    for _ in range(iterations):
        start = time.perf_counter()
        _ = feed.stop_times  # Access stop_times
        access_times.append(time.perf_counter() - start)

    print(f"stop_times access (cached):")
    print(f"  Mean: {sum(access_times) / len(access_times) * 1000:.4f} ms")
    print()

    # ============================================
    # Lazy loading (LazyGtfsFeed)
    # ============================================
    print("=" * 60)
    print("LazyGtfsFeed (lazy, defers parsing)")
    print("=" * 60)

    # Warmup
    _ = LazyGtfsFeed.from_path(str(GTFS_DIR))

    lazy_load_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        lazy_feed = LazyGtfsFeed.from_path(str(GTFS_DIR))
        lazy_load_times.append(time.perf_counter() - start)

    print(f"Load time (just scans directory):")
    print(f"  Mean: {sum(lazy_load_times) / len(lazy_load_times) * 1000:.2f} ms")
    print(f"  Min:  {min(lazy_load_times) * 1000:.2f} ms")

    # Count access (fast, counts CSV rows)
    lazy_feed = LazyGtfsFeed.from_path(str(GTFS_DIR))
    count_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        _ = lazy_feed.stop_time_count
        count_times.append(time.perf_counter() - start)

    print(f"stop_time_count (counts rows):")
    print(f"  Mean: {sum(count_times) / len(count_times) * 1000:.2f} ms")
    print(f"  Min:  {min(count_times) * 1000:.2f} ms")

    # First access (triggers parse)
    lazy_feed = LazyGtfsFeed.from_path(str(GTFS_DIR))
    first_access_times = []
    for _ in range(iterations):
        lazy_feed = LazyGtfsFeed.from_path(str(GTFS_DIR))  # Fresh feed each time
        start = time.perf_counter()
        _ = lazy_feed.stop_times
        first_access_times.append(time.perf_counter() - start)

    print(f"stop_times first access (parses stop_times.txt):")
    print(f"  Mean: {sum(first_access_times) / len(first_access_times) * 1000:.2f} ms")
    print(f"  Min:  {min(first_access_times) * 1000:.2f} ms")

    # Cached access
    lazy_feed = LazyGtfsFeed.from_path(str(GTFS_DIR))
    _ = lazy_feed.stop_times  # Populate cache
    cached_access_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        _ = lazy_feed.stop_times
        cached_access_times.append(time.perf_counter() - start)

    print(f"stop_times cached access:")
    print(f"  Mean: {sum(cached_access_times) / len(cached_access_times) * 1000:.4f} ms")
    print()

    # ============================================
    # Comparison with partridge
    # ============================================
    try:
        import partridge

        print("=" * 60)
        print("partridge (lazy loading)")
        print("=" * 60)

        partridge_load_times = []
        for _ in range(iterations):
            start = time.perf_counter()
            p_feed = partridge.load_geo_feed(str(GTFS_DIR))
            partridge_load_times.append(time.perf_counter() - start)

        print(f"Load time:")
        print(f"  Mean: {sum(partridge_load_times) / len(partridge_load_times) * 1000:.2f} ms")
        print(f"  Min:  {min(partridge_load_times) * 1000:.2f} ms")

        # First access
        partridge_first_times = []
        for _ in range(iterations):
            p_feed = partridge.load_geo_feed(str(GTFS_DIR))
            start = time.perf_counter()
            _ = p_feed.stop_times
            partridge_first_times.append(time.perf_counter() - start)

        print(f"stop_times first access:")
        print(f"  Mean: {sum(partridge_first_times) / len(partridge_first_times) * 1000:.2f} ms")
        print(f"  Min:  {min(partridge_first_times) * 1000:.2f} ms")
        print()

    except ImportError:
        print("partridge not installed, skipping comparison")
        print()

    # ============================================
    # DataFrame benchmark
    # ============================================
    print("=" * 60)
    print("GtfsDataFrames (pandas)")
    print("=" * 60)

    df_first_times = []
    try:
        from transit_parser.dataframes import GtfsDataFrames

        df_load_times = []
        for _ in range(iterations):
            start = time.perf_counter()
            dfs = GtfsDataFrames.from_path(str(GTFS_DIR))
            df_load_times.append(time.perf_counter() - start)

        print(f"Load time (just creates wrapper):")
        print(f"  Mean: {sum(df_load_times) / len(df_load_times) * 1000:.2f} ms")
        print(f"  Min:  {min(df_load_times) * 1000:.2f} ms")

        # First DataFrame access
        for _ in range(iterations):
            dfs = GtfsDataFrames.from_path(str(GTFS_DIR))
            start = time.perf_counter()
            _ = dfs.stop_times
            df_first_times.append(time.perf_counter() - start)

        print(f"stop_times DataFrame first access:")
        print(f"  Mean: {sum(df_first_times) / len(df_first_times) * 1000:.2f} ms")
        print(f"  Min:  {min(df_first_times) * 1000:.2f} ms")
        print()

    except ImportError:
        print("pandas not installed, skipping DataFrame benchmark")
        print()

    # ============================================
    # Summary
    # ============================================
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)

    eager_mean = sum(load_times) / len(load_times)
    lazy_mean = sum(lazy_load_times) / len(lazy_load_times)
    first_access_mean = sum(first_access_times) / len(first_access_times)

    print(f"GtfsFeed (eager):")
    print(f"  Total time to first data: {eager_mean * 1000:.2f} ms")
    print()
    print(f"LazyGtfsFeed:")
    print(f"  Load only:                {lazy_mean * 1000:.2f} ms")
    print(f"  Load + first access:      {(lazy_mean + first_access_mean) * 1000:.2f} ms")
    print(f"  Speedup (load only):      {eager_mean / lazy_mean:.1f}x faster")
    print()

    if df_first_times:
        df_mean = sum(df_first_times) / len(df_first_times)
        print(f"GtfsDataFrames:")
        print(f"  Time to DataFrame:        {df_mean * 1000:.2f} ms")
        print()

    if 'partridge_load_times' in dir():
        partridge_mean = sum(partridge_load_times) / len(partridge_load_times)
        partridge_first_mean = sum(partridge_first_times) / len(partridge_first_times)
        print(f"vs partridge:")
        print(f"  Our load / partridge:     {lazy_mean / partridge_mean:.2f}x")
        print(f"  Our first access / theirs: {first_access_mean / partridge_first_mean:.2f}x")
        if df_first_times:
            print(f"  Our DataFrame / partridge: {df_mean / partridge_first_mean:.2f}x")


if __name__ == "__main__":
    run_benchmark()
