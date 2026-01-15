# API Reference

This section provides detailed API documentation for all transit-parser modules.

## Core Classes

### GTFS

- [GtfsFeed](gtfs.md#gtfsfeed) - Eager-loading GTFS feed
- [LazyGtfsFeed](gtfs.md#lazygtfsfeed) - Lazy-loading GTFS feed

### TXC

- [TxcDocument](txc.md#txcdocument) - TransXChange document
- [TxcToGtfsConverter](txc.md#txctogtfsconverter) - TXC to GTFS converter

### Filtering

- [GtfsFilter](filtering.md#gtfsfilter) - Filtering and querying API

## Data Models

All GTFS entities are exposed as Python classes:

| Class | Description |
|-------|-------------|
| `Agency` | Transit agency |
| `Stop` | Transit stop/station |
| `Route` | Transit route |
| `Trip` | Individual trip |
| `StopTime` | Arrival/departure at a stop |
| `Calendar` | Service schedule |
| `CalendarDate` | Service exception |
| `Shape` | Route geometry |

## Exceptions

See [Exceptions](exceptions.md) for the complete exception hierarchy:

- `TransitParserError` - Base exception
- `GtfsError` - GTFS-related errors
- `TxcError` - TXC-related errors
- `ConversionError` - Conversion errors
- `FilterError` - Filtering errors

## Module Structure

```
transit_parser
├── GtfsFeed
├── LazyGtfsFeed
├── TxcDocument
├── TxcToGtfsConverter
├── ConversionOptions
├── ConversionResult
├── ConversionStats
├── Agency, Stop, Route, Trip, StopTime, Calendar, CalendarDate, Shape
├── TransitParserError, GtfsError, TxcError, ...
├── filtering
│   └── GtfsFilter
└── dataframes
    └── GtfsDataFrames
```
