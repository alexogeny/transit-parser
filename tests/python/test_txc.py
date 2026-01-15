"""Unit tests for TXC document parsing."""

from __future__ import annotations

from pathlib import Path

import pytest


class TestTxcDocument:
    """Tests for the TxcDocument class."""

    def test_load_from_file(self, txc_fixtures_dir: Path) -> None:
        """Test loading a TXC document from a file."""
        from transit_parser import TxcDocument

        txc_file = txc_fixtures_dir / "sample_service.xml"
        doc = TxcDocument.from_path(str(txc_file))
        assert doc is not None

    def test_schema_version(self, sample_txc_document) -> None:
        """Test that schema version is parsed correctly."""
        assert sample_txc_document.schema_version == "2.4"

    def test_filename(self, sample_txc_document) -> None:
        """Test that filename is available."""
        assert sample_txc_document.filename == "sample_service.xml"

    def test_operator_count(self, sample_txc_document) -> None:
        """Test that operator count is correct."""
        # operator_count is a property (int), not a method
        assert sample_txc_document.operator_count == 1

    def test_operator_names(self, sample_txc_document) -> None:
        """Test that operator names are accessible."""
        names = sample_txc_document.get_operator_names()
        assert len(names) == 1
        assert "Sample Bus" in names[0]

    def test_service_count(self, sample_txc_document) -> None:
        """Test that service count is correct."""
        assert sample_txc_document.service_count == 1

    def test_service_codes(self, sample_txc_document) -> None:
        """Test that service codes are accessible."""
        codes = sample_txc_document.get_service_codes()
        assert "SVC001" in codes

    def test_stop_point_count(self, sample_txc_document) -> None:
        """Test that stop point count is correct."""
        assert sample_txc_document.stop_point_count == 4

    def test_stop_codes(self, sample_txc_document) -> None:
        """Test that stop codes are accessible."""
        codes = sample_txc_document.get_stop_codes()
        assert len(codes) == 4
        assert "0100BRP90310" in codes
        assert "0100BRP90313" in codes

    def test_vehicle_journey_count(self, sample_txc_document) -> None:
        """Test that vehicle journey count is correct."""
        assert sample_txc_document.vehicle_journey_count == 5

    def test_journey_pattern_section_count(self, sample_txc_document) -> None:
        """Test that journey pattern section count is correct."""
        assert sample_txc_document.journey_pattern_section_count == 2


class TestTxcDocumentEdgeCases:
    """Tests for edge cases in TXC document parsing."""

    def test_nonexistent_path_raises_error(self) -> None:
        """Test that loading from a nonexistent path raises an error."""
        from transit_parser import TxcDocument

        with pytest.raises(Exception):
            TxcDocument.from_path("/nonexistent/path")

    def test_invalid_xml_returns_empty_document(self, tmp_path: Path) -> None:
        """Test that loading invalid XML returns an empty document."""
        from transit_parser import TxcDocument

        invalid_file = tmp_path / "invalid.xml"
        invalid_file.write_text("this is not valid xml")

        # Parser returns an empty document instead of raising
        doc = TxcDocument.from_path(str(invalid_file))
        assert doc.operator_count == 0
        assert doc.service_count == 0
        assert doc.schema_version == ""

    def test_empty_xml_returns_empty_document(self, tmp_path: Path) -> None:
        """Test that loading empty XML returns an empty document."""
        from transit_parser import TxcDocument

        empty_file = tmp_path / "empty.xml"
        empty_file.write_text("")

        # Parser returns an empty document instead of raising
        doc = TxcDocument.from_path(str(empty_file))
        assert doc.operator_count == 0
        assert doc.service_count == 0


class TestTxcDocumentFromString:
    """Tests for loading TXC from string content."""

    def test_from_string(self) -> None:
        """Test loading TXC document from string."""
        from transit_parser import TxcDocument

        xml_content = '''<?xml version="1.0" encoding="UTF-8"?>
<TransXChange xmlns="http://www.transxchange.org.uk/" SchemaVersion="2.4">
  <Operators>
    <Operator id="OP1">
      <OperatorShortName>Test Operator</OperatorShortName>
    </Operator>
  </Operators>
  <Services>
    <Service>
      <ServiceCode>TEST001</ServiceCode>
    </Service>
  </Services>
  <StopPoints></StopPoints>
  <VehicleJourneys></VehicleJourneys>
</TransXChange>'''

        doc = TxcDocument.from_string(xml_content)
        assert doc is not None
        assert doc.operator_count == 1
        assert doc.service_count == 1
