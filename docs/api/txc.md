# TXC API Reference

## TxcDocument

A parsed TransXChange document.

### Loading Methods

#### from_path
```python
TxcDocument.from_path(path: str) -> TxcDocument
```
Load a TXC document from an XML file.

#### from_string
```python
TxcDocument.from_string(xml: str) -> TxcDocument
```
Load a TXC document from an XML string.

#### from_bytes
```python
TxcDocument.from_bytes(data: bytes) -> TxcDocument
```
Load a TXC document from XML bytes.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `schema_version` | `str` | Schema version ("2.4" or "2.5") |
| `filename` | `Optional[str]` | Source filename |
| `operator_count` | `int` | Number of operators |
| `service_count` | `int` | Number of services |
| `stop_point_count` | `int` | Number of stop points |
| `vehicle_journey_count` | `int` | Number of vehicle journeys |
| `journey_pattern_section_count` | `int` | Number of journey pattern sections |

### Query Methods

#### get_operator_names
```python
doc.get_operator_names() -> List[str]
```
Get the names of all operators in the document.

#### get_service_codes
```python
doc.get_service_codes() -> List[str]
```
Get all service codes in the document.

#### get_stop_codes
```python
doc.get_stop_codes() -> List[str]
```
Get all stop codes (NaPTAN ATCOcodes) in the document.

---

## TxcToGtfsConverter

Converts TXC documents to GTFS format.

### Constructor

```python
TxcToGtfsConverter(options: Optional[ConversionOptions] = None)
```

### Methods

#### convert
```python
converter.convert(document: TxcDocument) -> ConversionResult
```
Convert a single TXC document to GTFS.

#### convert_batch
```python
converter.convert_batch(documents: List[TxcDocument]) -> ConversionResult
```
Convert multiple TXC documents into a single merged GTFS feed.

---

## ConversionOptions

Options for TXC to GTFS conversion.

### Constructor

```python
ConversionOptions(
    include_shapes: bool = False,
    calendar_start: Optional[str] = None,
    calendar_end: Optional[str] = None,
    region: str = "england",
    default_timezone: str = "Europe/London",
    default_agency_url: str = "https://example.com",
)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `include_shapes` | `bool` | `False` | Generate shapes from route sections |
| `calendar_start` | `Optional[str]` | `None` | Override calendar start (YYYY-MM-DD) |
| `calendar_end` | `Optional[str]` | `None` | Override calendar end (YYYY-MM-DD) |
| `region` | `str` | `"england"` | UK region for bank holidays |
| `default_timezone` | `str` | `"Europe/London"` | Default timezone |
| `default_agency_url` | `str` | `"https://example.com"` | Default agency URL |

---

## ConversionResult

Result of a TXC to GTFS conversion.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `feed` | `GtfsFeed` | The converted GTFS feed |
| `warnings` | `List[str]` | Warnings generated during conversion |
| `stats` | `ConversionStats` | Conversion statistics |

---

## ConversionStats

Statistics from a conversion.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `agencies_converted` | `int` | Number of agencies converted |
| `stops_converted` | `int` | Number of stops converted |
| `routes_converted` | `int` | Number of routes converted |
| `trips_converted` | `int` | Number of trips converted |
| `stop_times_generated` | `int` | Number of stop times generated |
| `calendar_entries` | `int` | Number of calendar entries |
| `calendar_exceptions` | `int` | Number of calendar exceptions |
| `shapes_generated` | `int` | Number of shapes generated |
