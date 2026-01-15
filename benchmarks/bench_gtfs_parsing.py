"""Benchmarks for GTFS parsing comparing transit-parser against other libraries."""

import time
from pathlib import Path
import tempfile

import pytest

# Our library
from transit_parser import GtfsFeed

# Test file - we'll use the output from TXC conversion
OUTPUT_ZIP = Path(__file__).parent.parent / "output.zip"


def check_file_exists():
    """Check if test file exists."""
    if not OUTPUT_ZIP.exists():
        pytest.skip(f"Test file not found: {OUTPUT_ZIP}. Run bench_txc_parsing.py first.")


class TestTransitParserGtfsBenchmarks:
    """Benchmarks for transit-parser GTFS operations."""

    def test_gtfs_from_zip(self, benchmark):
        """Benchmark GTFS ZIP loading."""
        check_file_exists()
        result = benchmark(GtfsFeed.from_zip, str(OUTPUT_ZIP))
        assert len(result.stops) > 0
        assert len(result.trips) > 0

    def test_gtfs_to_zip(self, benchmark):
        """Benchmark GTFS ZIP writing."""
        check_file_exists()
        feed = GtfsFeed.from_zip(str(OUTPUT_ZIP))

        with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as f:
            output_path = f.name

        def write_zip():
            feed.to_zip(output_path)

        benchmark(write_zip)
        Path(output_path).unlink(missing_ok=True)

    def test_gtfs_roundtrip(self, benchmark):
        """Benchmark full load + save roundtrip."""
        check_file_exists()

        with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as f:
            output_path = f.name

        def roundtrip():
            feed = GtfsFeed.from_zip(str(OUTPUT_ZIP))
            feed.to_zip(output_path)
            return feed

        result = benchmark(roundtrip)
        assert len(result.stops) > 0
        Path(output_path).unlink(missing_ok=True)


# Optional: Compare against gtfs-kit if available
try:
    import gtfs_kit

    class TestGtfsKitComparison:
        """Comparison benchmarks against gtfs-kit library."""

        def test_gtfs_kit_read(self, benchmark):
            """Benchmark gtfs-kit feed loading."""
            check_file_exists()
            result = benchmark(gtfs_kit.read_feed, str(OUTPUT_ZIP), dist_units="km")
            assert result.stops is not None

except ImportError:
    pass


# Optional: Compare against partridge if available
try:
    import partridge as ptg

    class TestPartridgeComparison:
        """Comparison benchmarks against partridge library."""

        def test_partridge_load(self, benchmark):
            """Benchmark partridge feed loading."""
            check_file_exists()
            result = benchmark(ptg.load_geo_feed, str(OUTPUT_ZIP))
            assert len(result.stops) > 0

except ImportError:
    pass


# Manual timing for quick comparison
def run_manual_benchmark():
    """Run a manual benchmark without pytest-benchmark."""
    if not OUTPUT_ZIP.exists():
        print(f"Test file not found: {OUTPUT_ZIP}")
        print("Run bench_txc_parsing.py first to generate the GTFS output.")
        return

    print(f"Benchmarking with file: {OUTPUT_ZIP}")
    print(f"File size: {OUTPUT_ZIP.stat().st_size / 1024 / 1024:.2f} MB")
    print()

    # Warm up
    _ = GtfsFeed.from_zip(str(OUTPUT_ZIP))

    iterations = 5

    # Read benchmark
    read_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        feed = GtfsFeed.from_zip(str(OUTPUT_ZIP))
        read_times.append(time.perf_counter() - start)

    print("transit-parser GTFS Read:")
    print(f"  Mean: {sum(read_times) / len(read_times) * 1000:.2f} ms")
    print(f"  Min:  {min(read_times) * 1000:.2f} ms")
    print(f"  Max:  {max(read_times) * 1000:.2f} ms")
    print()

    # Write benchmark
    with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as f:
        output_path = f.name

    write_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        feed.to_zip(output_path)
        write_times.append(time.perf_counter() - start)

    print("transit-parser GTFS Write:")
    print(f"  Mean: {sum(write_times) / len(write_times) * 1000:.2f} ms")
    print(f"  Min:  {min(write_times) * 1000:.2f} ms")
    print(f"  Max:  {max(write_times) * 1000:.2f} ms")
    print()

    Path(output_path).unlink(missing_ok=True)

    # Stats
    print("Feed Stats:")
    print(f"  Agencies:    {len(feed.agencies)}")
    print(f"  Stops:       {len(feed.stops)}")
    print(f"  Routes:      {len(feed.routes)}")
    print(f"  Trips:       {len(feed.trips)}")
    print(f"  Stop times:  {len(feed.stop_times)}")
    print()

    # Compare with other libraries if available
    try:
        import gtfs_kit

        print("Comparing with gtfs-kit...")
        gtfs_kit_times = []
        for _ in range(iterations):
            start = time.perf_counter()
            _ = gtfs_kit.read_feed(str(OUTPUT_ZIP), dist_units="km")
            gtfs_kit_times.append(time.perf_counter() - start)

        print("gtfs-kit Read:")
        print(f"  Mean: {sum(gtfs_kit_times) / len(gtfs_kit_times) * 1000:.2f} ms")
        print(f"  Min:  {min(gtfs_kit_times) * 1000:.2f} ms")
        print()
        print(
            f"Speedup: {sum(gtfs_kit_times) / sum(read_times):.1f}x faster than gtfs-kit"
        )
    except ImportError:
        print("gtfs-kit not installed, skipping comparison")
        print("  Install with: pip install gtfs-kit")
    print()

    try:
        import partridge as ptg

        print("Comparing with partridge...")
        partridge_times = []
        for _ in range(iterations):
            start = time.perf_counter()
            _ = ptg.load_feed(str(OUTPUT_ZIP))
            partridge_times.append(time.perf_counter() - start)

        print("partridge Read:")
        print(f"  Mean: {sum(partridge_times) / len(partridge_times) * 1000:.2f} ms")
        print(f"  Min:  {min(partridge_times) * 1000:.2f} ms")
        print()
        print(
            f"Speedup: {sum(partridge_times) / sum(read_times):.1f}x faster than partridge"
        )
    except ImportError:
        print("partridge not installed, skipping comparison")
        print("  Install with: pip install partridge")


if __name__ == "__main__":
    run_manual_benchmark()
