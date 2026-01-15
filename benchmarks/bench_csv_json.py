"""Benchmarks for CSV and JSON parsing."""

import json
import time
from pathlib import Path
import tempfile
import csv as csv_stdlib

import pytest

# Our library
from transit_parser.io import CsvDocument, JsonDocument


def create_test_csv(rows: int = 10000) -> str:
    """Create a test CSV file with the specified number of rows."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".csv", delete=False) as f:
        writer = csv_stdlib.writer(f)
        writer.writerow(["id", "name", "value", "category", "timestamp"])
        for i in range(rows):
            writer.writerow([i, f"item_{i}", i * 1.5, f"cat_{i % 10}", f"2024-01-{(i % 28) + 1:02d}"])
        return f.name


def create_test_json(items: int = 10000) -> str:
    """Create a test JSON file with the specified number of items."""
    data = {
        "items": [
            {"id": i, "name": f"item_{i}", "value": i * 1.5, "tags": [f"tag_{j}" for j in range(3)]}
            for i in range(items)
        ],
        "metadata": {"count": items, "version": "1.0"},
    }
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(data, f)
        return f.name


class TestCsvBenchmarks:
    """Benchmarks for CSV parsing."""

    @pytest.fixture(scope="class")
    def csv_file(self):
        """Create test CSV file."""
        path = create_test_csv(50000)
        yield path
        Path(path).unlink(missing_ok=True)

    def test_csv_parse(self, benchmark, csv_file):
        """Benchmark CSV parsing."""
        result = benchmark(CsvDocument.from_path, csv_file)
        assert len(result) > 0

    def test_csv_stdlib(self, benchmark, csv_file):
        """Benchmark standard library CSV parsing."""

        def parse_csv():
            with open(csv_file, "r") as f:
                reader = csv_stdlib.DictReader(f)
                return list(reader)

        result = benchmark(parse_csv)
        assert len(result) > 0


class TestJsonBenchmarks:
    """Benchmarks for JSON parsing."""

    @pytest.fixture(scope="class")
    def json_file(self):
        """Create test JSON file."""
        path = create_test_json(50000)
        yield path
        Path(path).unlink(missing_ok=True)

    def test_json_parse(self, benchmark, json_file):
        """Benchmark JSON parsing."""
        result = benchmark(JsonDocument.from_path, json_file)
        assert result.is_object()

    def test_json_stdlib(self, benchmark, json_file):
        """Benchmark standard library JSON parsing."""

        def parse_json():
            with open(json_file, "r") as f:
                return json.load(f)

        result = benchmark(parse_json)
        assert "items" in result


# Manual benchmarks
def run_manual_benchmark():
    """Run manual benchmarks for CSV and JSON."""
    iterations = 5

    # CSV benchmark
    print("Creating test CSV file (50,000 rows)...")
    csv_path = create_test_csv(50000)
    csv_size = Path(csv_path).stat().st_size / 1024 / 1024
    print(f"CSV file size: {csv_size:.2f} MB")
    print()

    # Warm up
    _ = CsvDocument.from_path(csv_path)

    # Our library
    our_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        doc = CsvDocument.from_path(csv_path)
        our_times.append(time.perf_counter() - start)

    print("transit-parser CSV Read:")
    print(f"  Mean: {sum(our_times) / len(our_times) * 1000:.2f} ms")
    print(f"  Rows: {len(doc)}")
    print()

    # Standard library
    stdlib_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        with open(csv_path, "r") as f:
            reader = csv_stdlib.DictReader(f)
            rows = list(reader)
        stdlib_times.append(time.perf_counter() - start)

    print("Python stdlib CSV Read:")
    print(f"  Mean: {sum(stdlib_times) / len(stdlib_times) * 1000:.2f} ms")
    print(f"  Rows: {len(rows)}")
    print()
    print(f"CSV Speedup: {sum(stdlib_times) / sum(our_times):.1f}x vs stdlib")
    print()

    Path(csv_path).unlink()

    # JSON benchmark
    print("Creating test JSON file (50,000 items)...")
    json_path = create_test_json(50000)
    json_size = Path(json_path).stat().st_size / 1024 / 1024
    print(f"JSON file size: {json_size:.2f} MB")
    print()

    # Warm up
    _ = JsonDocument.from_path(json_path)

    # Our library
    our_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        doc = JsonDocument.from_path(json_path)
        our_times.append(time.perf_counter() - start)

    print("transit-parser JSON Read:")
    print(f"  Mean: {sum(our_times) / len(our_times) * 1000:.2f} ms")
    print()

    # Standard library
    stdlib_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        with open(json_path, "r") as f:
            data = json.load(f)
        stdlib_times.append(time.perf_counter() - start)

    print("Python stdlib JSON Read:")
    print(f"  Mean: {sum(stdlib_times) / len(stdlib_times) * 1000:.2f} ms")
    print()
    print(f"JSON Speedup: {sum(stdlib_times) / sum(our_times):.1f}x vs stdlib")

    Path(json_path).unlink()


if __name__ == "__main__":
    run_manual_benchmark()
