# Parsing TransXChange (TXC)

TransXChange is the UK standard for bus timetable data. Transit Parser supports TXC schema versions 2.4 and 2.5.

## Loading TXC Documents

### From File

```python
from transit_parser import TxcDocument

doc = TxcDocument.from_path("service.xml")
```

### From String

```python
xml_content = """<?xml version="1.0"?>
<TransXChange xmlns="http://www.transxchange.org.uk/" SchemaVersion="2.4">
  ...
</TransXChange>
"""

doc = TxcDocument.from_string(xml_content)
```

### From Bytes

```python
with open("service.xml", "rb") as f:
    data = f.read()

doc = TxcDocument.from_bytes(data)
```

## Accessing Document Info

```python
# Schema version
print(doc.schema_version)  # "2.4" or "2.5"

# Filename (if loaded from file)
print(doc.filename)

# Entity counts
print(f"Operators: {doc.operator_count}")
print(f"Services: {doc.service_count}")
print(f"Stops: {doc.stop_point_count}")
print(f"Journeys: {doc.vehicle_journey_count}")
print(f"Patterns: {doc.journey_pattern_section_count}")
```

## Querying Document Data

```python
# Get operator names
operators = doc.get_operator_names()
print(f"Operators: {operators}")

# Get service codes
services = doc.get_service_codes()
print(f"Services: {services}")

# Get stop codes (NaPTAN ATCOcodes)
stops = doc.get_stop_codes()
print(f"Stops: {stops}")
```

## TXC Structure Overview

A TransXChange document contains:

| Element | Description |
|---------|-------------|
| Operators | Bus companies operating the services |
| Services | Service definitions with lines |
| StopPoints | Bus stops with NaPTAN references |
| Routes | Physical routes between stops |
| RouteSections | Segments of routes |
| JourneyPatternSections | Timing patterns |
| VehicleJourneys | Individual trip instances |

## Handling Invalid Documents

Transit Parser handles invalid XML gracefully:

```python
# Invalid XML returns an empty document
doc = TxcDocument.from_string("not valid xml")
print(doc.operator_count)  # 0
print(doc.schema_version)  # ""
```

To check if a document loaded successfully:

```python
doc = TxcDocument.from_path("service.xml")

if doc.operator_count == 0 and doc.service_count == 0:
    print("Warning: Document may be empty or invalid")
```

## Next Steps

See [TXC to GTFS Conversion](conversion.md) to convert TXC documents to GTFS format.
