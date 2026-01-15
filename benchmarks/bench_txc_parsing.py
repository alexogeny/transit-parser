"""Benchmarks for TXC parsing comparing transit-parser against other libraries."""

import os
import tempfile
import time
from pathlib import Path

import pytest

# Our library
from transit_parser import TxcDocument, TxcToGtfsConverter, ConversionOptions

# Test file - set via BENCH_TXC_FILE env var or use default
TXC_FILE = Path(os.environ.get("BENCH_TXC_FILE", Path(__file__).parent.parent / "sample.xml"))


def check_file_exists():
    """Check if test file exists."""
    if not TXC_FILE.exists():
        pytest.skip(f"Test file not found: {TXC_FILE}")


class TestTransitParserBenchmarks:
    """Benchmarks for transit-parser (our Rust-backed library)."""

    def test_txc_parse(self, benchmark):
        """Benchmark TXC document parsing."""
        check_file_exists()
        result = benchmark(TxcDocument.from_path, str(TXC_FILE))
        assert result.service_count > 0
        assert result.vehicle_journey_count > 0

    def test_txc_to_gtfs_conversion(self, benchmark):
        """Benchmark TXC to GTFS conversion."""
        check_file_exists()
        doc = TxcDocument.from_path(str(TXC_FILE))
        options = ConversionOptions(
            include_shapes=False,
            region="england",
            calendar_start="2025-04-28",
            calendar_end="2025-12-31",
        )
        converter = TxcToGtfsConverter(options)

        result = benchmark(converter.convert, doc)
        assert result.stats.trips_converted > 0
        assert result.stats.stop_times_generated > 0

    def test_full_pipeline(self, benchmark):
        """Benchmark full parse + convert pipeline."""
        check_file_exists()

        def full_pipeline():
            doc = TxcDocument.from_path(str(TXC_FILE))
            options = ConversionOptions(
                include_shapes=False,
                region="england",
            )
            converter = TxcToGtfsConverter(options)
            return converter.convert(doc)

        result = benchmark(full_pipeline)
        assert result.stats.trips_converted > 0


# Compare against transx2gtfs if available
try:
    import transx2gtfs

    class TestTransx2GtfsBenchmarks:
        """Benchmarks for transx2gtfs library."""

        def test_transx2gtfs_parse_and_convert(self, benchmark):
            """Benchmark transx2gtfs full pipeline."""
            check_file_exists()

            def parse_and_convert():
                with tempfile.TemporaryDirectory() as tmpdir:
                    output_path = Path(tmpdir) / "output.zip"
                    try:
                        transx2gtfs.convert(str(TXC_FILE), str(output_path))
                        return output_path.exists()
                    except Exception as e:
                        # transx2gtfs may fail on some files
                        return str(e)

            result = benchmark(parse_and_convert)

except ImportError:
    pass


# Compare against lxml for raw XML parsing baseline
try:
    from lxml import etree

    class TestLxmlBaseline:
        """Baseline comparison using lxml for XML parsing."""

        def test_lxml_parse(self, benchmark):
            """Benchmark raw XML parsing with lxml."""
            check_file_exists()

            def parse_xml():
                tree = etree.parse(str(TXC_FILE))
                root = tree.getroot()
                # Count some elements to ensure parsing happened
                ns = {"txc": "http://www.transxchange.org.uk/"}
                services = root.findall(".//txc:Service", ns) or root.findall(".//Service")
                return len(services) if services else len(root)

            result = benchmark(parse_xml)
            assert result > 0

except ImportError:
    pass


# Manual timing for quick comparison
def run_manual_benchmark():
    """Run a manual benchmark without pytest-benchmark."""
    if not TXC_FILE.exists():
        print(f"Test file not found: {TXC_FILE}")
        return

    print(f"Benchmarking with file: {TXC_FILE}")
    print(f"File size: {TXC_FILE.stat().st_size / 1024 / 1024:.2f} MB")
    print()

    iterations = 5

    # ============================================
    # transit-parser benchmarks
    # ============================================
    print("=" * 60)
    print("transit-parser (Rust)")
    print("=" * 60)

    # Warm up
    _ = TxcDocument.from_path(str(TXC_FILE))

    # Parse benchmark
    parse_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        doc = TxcDocument.from_path(str(TXC_FILE))
        parse_times.append(time.perf_counter() - start)

    print("TXC Parsing:")
    print(f"  Mean: {sum(parse_times) / len(parse_times) * 1000:.2f} ms")
    print(f"  Min:  {min(parse_times) * 1000:.2f} ms")

    # Conversion benchmark
    options = ConversionOptions(
        include_shapes=False,
        region="england",
        calendar_start="2025-04-28",
        calendar_end="2025-12-31",
    )
    converter = TxcToGtfsConverter(options)

    convert_times = []
    for _ in range(iterations):
        doc = TxcDocument.from_path(str(TXC_FILE))
        start = time.perf_counter()
        result = converter.convert(doc)
        convert_times.append(time.perf_counter() - start)

    print("TXC to GTFS Conversion:")
    print(f"  Mean: {sum(convert_times) / len(convert_times) * 1000:.2f} ms")
    print(f"  Min:  {min(convert_times) * 1000:.2f} ms")

    # Full pipeline benchmark
    full_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        doc = TxcDocument.from_path(str(TXC_FILE))
        result = converter.convert(doc)
        full_times.append(time.perf_counter() - start)

    print("Full Pipeline (Parse + Convert):")
    print(f"  Mean: {sum(full_times) / len(full_times) * 1000:.2f} ms")
    print(f"  Min:  {min(full_times) * 1000:.2f} ms")

    our_full_mean = sum(full_times) / len(full_times)

    print()
    print("Conversion Results:")
    print(f"  Services:    {doc.service_count}")
    print(f"  Stops:       {doc.stop_point_count}")
    print(f"  Trips:       {result.stats.trips_converted}")
    print(f"  Stop times:  {result.stats.stop_times_generated}")
    print()

    # ============================================
    # transx2gtfs benchmarks
    # ============================================
    try:
        import transx2gtfs

        print("=" * 60)
        print("transx2gtfs (Python)")
        print("=" * 60)

        transx2gtfs_times = []
        for i in range(iterations):
            with tempfile.TemporaryDirectory() as tmpdir:
                output_path = Path(tmpdir) / "output.zip"
                start = time.perf_counter()
                try:
                    transx2gtfs.convert(str(TXC_FILE), str(output_path))
                    transx2gtfs_times.append(time.perf_counter() - start)
                except Exception as e:
                    if i == 0:
                        print(f"  Error: {e}")
                    transx2gtfs_times.append(time.perf_counter() - start)

        if transx2gtfs_times:
            transx2gtfs_mean = sum(transx2gtfs_times) / len(transx2gtfs_times)
            print("Full Pipeline (Parse + Convert):")
            print(f"  Mean: {transx2gtfs_mean * 1000:.2f} ms")
            print(f"  Min:  {min(transx2gtfs_times) * 1000:.2f} ms")
            print()
            print(f"Speedup vs transx2gtfs: {transx2gtfs_mean / our_full_mean:.1f}x faster")
        print()

    except ImportError:
        print("transx2gtfs not installed, skipping comparison")
        print("  Install with: pip install transx2gtfs")
        print()

    # ============================================
    # lxml baseline (raw XML parsing)
    # ============================================
    try:
        from lxml import etree

        print("=" * 60)
        print("lxml baseline (raw XML parsing only)")
        print("=" * 60)

        lxml_times = []
        for _ in range(iterations):
            start = time.perf_counter()
            tree = etree.parse(str(TXC_FILE))
            root = tree.getroot()
            lxml_times.append(time.perf_counter() - start)

        lxml_mean = sum(lxml_times) / len(lxml_times)
        print("XML Parsing:")
        print(f"  Mean: {lxml_mean * 1000:.2f} ms")
        print(f"  Min:  {min(lxml_times) * 1000:.2f} ms")
        print()
        print(f"Our parse vs lxml: {sum(parse_times) / len(parse_times) / lxml_mean:.1f}x slower")
        print("  (Note: lxml only parses XML, we also extract structured data)")
        print()

    except ImportError:
        print("lxml not installed, skipping baseline")
        print()

    # ============================================
    # pytxc comparison (if available)
    # ============================================
    try:
        import pytxc

        print("=" * 60)
        print("pytxc (Python)")
        print("=" * 60)

        pytxc_times = []
        for _ in range(iterations):
            start = time.perf_counter()
            try:
                txc = pytxc.Timetable.from_file(str(TXC_FILE))
                pytxc_times.append(time.perf_counter() - start)
            except Exception as e:
                print(f"  Error: {e}")
                break

        if pytxc_times:
            pytxc_mean = sum(pytxc_times) / len(pytxc_times)
            print("TXC Parsing:")
            print(f"  Mean: {pytxc_mean * 1000:.2f} ms")
            print(f"  Min:  {min(pytxc_times) * 1000:.2f} ms")
            print()
            print(f"Speedup vs pytxc: {pytxc_mean / (sum(parse_times) / len(parse_times)):.1f}x faster")
        print()

    except ImportError:
        print("pytxc not installed (requires Python <3.12 due to shapely dependency)")
        print()


if __name__ == "__main__":
    run_manual_benchmark()
