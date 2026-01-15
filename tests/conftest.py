"""Pytest configuration and fixtures for transit-parser tests."""

from __future__ import annotations

import os
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from transit_parser import GtfsFeed, LazyGtfsFeed, TxcDocument


# Get the fixtures directory
FIXTURES_DIR = Path(__file__).parent / "fixtures"
GTFS_FIXTURES_DIR = FIXTURES_DIR / "gtfs"
TXC_FIXTURES_DIR = FIXTURES_DIR / "txc"


@pytest.fixture
def gtfs_fixtures_dir() -> Path:
    """Return the path to the GTFS fixtures directory."""
    return GTFS_FIXTURES_DIR


@pytest.fixture
def txc_fixtures_dir() -> Path:
    """Return the path to the TXC fixtures directory."""
    return TXC_FIXTURES_DIR


@pytest.fixture
def sample_gtfs_feed() -> "GtfsFeed":
    """Load the sample GTFS feed from fixtures."""
    from transit_parser import GtfsFeed

    return GtfsFeed.from_path(str(GTFS_FIXTURES_DIR))


@pytest.fixture
def sample_lazy_gtfs_feed() -> "LazyGtfsFeed":
    """Load the sample GTFS feed lazily from fixtures."""
    from transit_parser import LazyGtfsFeed

    return LazyGtfsFeed.from_path(str(GTFS_FIXTURES_DIR))


@pytest.fixture
def sample_txc_document() -> "TxcDocument":
    """Load the sample TXC document from fixtures."""
    from transit_parser import TxcDocument

    txc_file = TXC_FIXTURES_DIR / "sample_service.xml"
    return TxcDocument.from_path(str(txc_file))


@pytest.fixture
def temp_output_dir(tmp_path: Path) -> Path:
    """Create a temporary directory for test outputs."""
    output_dir = tmp_path / "output"
    output_dir.mkdir()
    return output_dir
