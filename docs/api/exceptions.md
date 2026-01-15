# Exceptions Reference

Transit Parser provides a hierarchy of exceptions for precise error handling.

## Exception Hierarchy

```
TransitParserError (base)
├── GtfsError
│   ├── GtfsFileNotFoundError
│   ├── GtfsValidationError
│   └── GtfsParseError
├── TxcError
│   ├── TxcFileNotFoundError
│   ├── TxcValidationError
│   └── TxcParseError
├── ConversionError
│   ├── MappingError
│   └── CalendarConversionError
└── FilterError
    └── InvalidDateError
```

## Base Exception

### TransitParserError

```python
class TransitParserError(Exception)
```

Base exception for all transit-parser errors. Catch this to handle any error from the library.

```python
from transit_parser import TransitParserError

try:
    # Any transit-parser operation
    ...
except TransitParserError as e:
    print(f"Transit parser error: {e}")
```

## GTFS Exceptions

### GtfsError

```python
class GtfsError(TransitParserError)
```

Base exception for GTFS-related errors.

### GtfsFileNotFoundError

```python
class GtfsFileNotFoundError(GtfsError)
```

Raised when a GTFS file or directory cannot be found.

**Attributes:**
- `path: str | None` - The path that was not found
- `missing_files: List[str]` - List of missing required files

```python
from transit_parser import GtfsFileNotFoundError

try:
    feed = GtfsFeed.from_path("/nonexistent")
except GtfsFileNotFoundError as e:
    print(f"Path: {e.path}")
    print(f"Missing files: {e.missing_files}")
```

### GtfsValidationError

```python
class GtfsValidationError(GtfsError)
```

Raised when GTFS data fails validation.

**Attributes:**
- `errors: List[str]` - Validation error messages
- `warnings: List[str]` - Validation warning messages

### GtfsParseError

```python
class GtfsParseError(GtfsError)
```

Raised when GTFS CSV data cannot be parsed.

**Attributes:**
- `file_name: str | None` - The file that failed to parse
- `line_number: int | None` - The line number of the error
- `column: str | None` - The column name where the error occurred

## TXC Exceptions

### TxcError

```python
class TxcError(TransitParserError)
```

Base exception for TXC-related errors.

### TxcFileNotFoundError

```python
class TxcFileNotFoundError(TxcError)
```

Raised when a TXC file cannot be found.

**Attributes:**
- `path: str | None` - The path that was not found

### TxcValidationError

```python
class TxcValidationError(TxcError)
```

Raised when TXC data fails validation.

**Attributes:**
- `schema_version: str | None` - The schema version of the document
- `errors: List[str]` - Validation error messages

### TxcParseError

```python
class TxcParseError(TxcError)
```

Raised when TXC XML cannot be parsed.

**Attributes:**
- `element: str | None` - The XML element where the error occurred
- `line_number: int | None` - The line number in the XML file

## Conversion Exceptions

### ConversionError

```python
class ConversionError(TransitParserError)
```

Base exception for conversion-related errors.

### MappingError

```python
class MappingError(ConversionError)
```

Raised when data cannot be mapped between formats.

**Attributes:**
- `source_type: str | None` - The source data type
- `target_type: str | None` - The target data type
- `field: str | None` - The field that failed to map

### CalendarConversionError

```python
class CalendarConversionError(ConversionError)
```

Raised when calendar/service data cannot be converted.

**Attributes:**
- `service_id: str | None` - The service ID that failed
- `reason: str | None` - The reason for the failure

## Filter Exceptions

### FilterError

```python
class FilterError(TransitParserError)
```

Base exception for filtering-related errors.

### InvalidDateError

```python
class InvalidDateError(FilterError)
```

Raised when an invalid date is provided for filtering.

**Attributes:**
- `date_string: str | None` - The invalid date string
- `expected_format: str | None` - The expected date format

```python
from transit_parser import InvalidDateError
from transit_parser.filtering import GtfsFilter

try:
    f.active_services_on("not-a-date")
except InvalidDateError as e:
    print(f"Invalid: {e.date_string}")
    print(f"Expected: {e.expected_format}")
```

## Best Practices

### Catch Specific Exceptions

```python
from transit_parser import (
    GtfsFileNotFoundError,
    GtfsParseError,
    InvalidDateError,
)

try:
    feed = GtfsFeed.from_path(path)
    services = GtfsFilter(feed).active_services_on(date)
except GtfsFileNotFoundError:
    print("Feed not found")
except GtfsParseError:
    print("Feed is malformed")
except InvalidDateError:
    print("Invalid date provided")
```

### Catch by Category

```python
from transit_parser import GtfsError, TxcError

try:
    # GTFS operations
    ...
except GtfsError:
    print("Problem with GTFS data")

try:
    # TXC operations
    ...
except TxcError:
    print("Problem with TXC data")
```

### Catch Everything

```python
from transit_parser import TransitParserError

try:
    # Any operation
    ...
except TransitParserError as e:
    print(f"Error: {e}")
```
