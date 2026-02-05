//! Export format presets.

use super::generic_csv::{ColumnConfig, ExportConfig, TimeFormat};

/// Predefined export format presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportPreset {
    /// Default transit-parser format.
    Default,
    /// Minimal format with essential columns.
    Minimal,
    /// Extended format with all available columns.
    Extended,
    /// Optibus-like format (approximate).
    OptibusLike,
    /// Hastus-like format (approximate).
    HastusLike,
    /// GTFS-style block format.
    GtfsBlock,
}

impl ExportPreset {
    /// Convert preset to ExportConfig.
    pub fn to_config(self) -> ExportConfig {
        match self {
            ExportPreset::Default => ExportConfig::default(),
            ExportPreset::Minimal => Self::minimal_config(),
            ExportPreset::Extended => Self::extended_config(),
            ExportPreset::OptibusLike => Self::optibus_like_config(),
            ExportPreset::HastusLike => Self::hastus_like_config(),
            ExportPreset::GtfsBlock => Self::gtfs_block_config(),
        }
    }

    fn minimal_config() -> ExportConfig {
        ExportConfig {
            columns: vec![
                ColumnConfig::new("run_number", "run_number"),
                ColumnConfig::new("block", "block"),
                ColumnConfig::new("trip_id", "trip_id"),
                ColumnConfig::new("start_time", "start_time"),
                ColumnConfig::new("end_time", "end_time"),
            ],
            ..Default::default()
        }
    }

    fn extended_config() -> ExportConfig {
        ExportConfig {
            columns: vec![
                ColumnConfig::new("run_number", "run_number"),
                ColumnConfig::new("duty_id", "duty_id"),
                ColumnConfig::new("shift_id", "shift_id"),
                ColumnConfig::new("block", "block"),
                ColumnConfig::new("start_place", "start_place"),
                ColumnConfig::new("end_place", "end_place"),
                ColumnConfig::new("start_time", "start_time"),
                ColumnConfig::new("end_time", "end_time"),
                ColumnConfig::new("trip_id", "trip_id"),
                ColumnConfig::new("route_short_name", "route_short_name"),
                ColumnConfig::new("headsign", "headsign"),
                ColumnConfig::new("depot", "depot"),
                ColumnConfig::new("vehicle_class", "vehicle_class"),
                ColumnConfig::new("vehicle_type", "vehicle_type"),
                ColumnConfig::new("start_lat", "start_lat"),
                ColumnConfig::new("start_lon", "start_lon"),
                ColumnConfig::new("end_lat", "end_lat"),
                ColumnConfig::new("end_lon", "end_lon"),
                ColumnConfig::new("route_shape_id", "route_shape_id"),
                ColumnConfig::new("row_type", "row_type"),
            ],
            ..Default::default()
        }
    }

    /// Optibus-like format.
    ///
    /// Note: This is an approximation based on common scheduling software patterns.
    /// Actual Optibus format is proprietary.
    fn optibus_like_config() -> ExportConfig {
        ExportConfig {
            columns: vec![
                ColumnConfig::new("run_number", "Run"),
                ColumnConfig::new("block", "Block"),
                ColumnConfig::new("row_type", "Activity"),
                ColumnConfig::new("start_place", "StartStop"),
                ColumnConfig::new("end_place", "EndStop"),
                ColumnConfig::new("start_time", "StartTime"),
                ColumnConfig::new("end_time", "EndTime"),
                ColumnConfig::new("trip_id", "TripID"),
                ColumnConfig::new("route_short_name", "Route"),
                ColumnConfig::new("headsign", "Direction"),
                ColumnConfig::new("depot", "Depot"),
                ColumnConfig::new("vehicle_type", "VehicleType"),
            ],
            time_format: TimeFormat::HhMmSs,
            ..Default::default()
        }
    }

    /// Hastus-like format.
    ///
    /// Note: This is an approximation based on common scheduling software patterns.
    /// Actual Hastus format is proprietary.
    fn hastus_like_config() -> ExportConfig {
        ExportConfig {
            columns: vec![
                ColumnConfig::new("duty_id", "DUTY_NO"),
                ColumnConfig::new("block", "BLOCK_NO"),
                ColumnConfig::new("run_number", "RUN_NO"),
                ColumnConfig::new("trip_id", "TRIP_NO"),
                ColumnConfig::new("route_short_name", "ROUTE"),
                ColumnConfig::new("start_place", "FROM_STOP"),
                ColumnConfig::new("end_place", "TO_STOP"),
                ColumnConfig::new("start_time", "START"),
                ColumnConfig::new("end_time", "END"),
                ColumnConfig::new("row_type", "TYPE"),
                ColumnConfig::new("depot", "GARAGE"),
                ColumnConfig::new("vehicle_class", "VEH_TYPE"),
            ],
            time_format: TimeFormat::HhMm,
            ..Default::default()
        }
    }

    /// GTFS-compatible block format.
    fn gtfs_block_config() -> ExportConfig {
        ExportConfig {
            columns: vec![
                ColumnConfig::new("block", "block_id"),
                ColumnConfig::new("trip_id", "trip_id"),
                ColumnConfig::new("start_time", "start_time"),
                ColumnConfig::new("end_time", "end_time"),
                ColumnConfig::new("start_place", "start_stop_id"),
                ColumnConfig::new("end_place", "end_stop_id"),
                ColumnConfig::new("route_shape_id", "shape_id"),
            ],
            time_format: TimeFormat::HhMmSs,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_to_config() {
        let config = ExportPreset::Minimal.to_config();
        assert_eq!(config.columns.len(), 5);

        let config = ExportPreset::Extended.to_config();
        assert!(config.columns.len() > 15);
    }

    #[test]
    fn test_optibus_like_column_names() {
        let config = ExportPreset::OptibusLike.to_config();

        let headers: Vec<&str> = config.columns.iter().map(|c| c.header.as_str()).collect();
        assert!(headers.contains(&"Run"));
        assert!(headers.contains(&"Block"));
        assert!(headers.contains(&"TripID"));
    }

    #[test]
    fn test_hastus_like_column_names() {
        let config = ExportPreset::HastusLike.to_config();

        let headers: Vec<&str> = config.columns.iter().map(|c| c.header.as_str()).collect();
        assert!(headers.contains(&"DUTY_NO"));
        assert!(headers.contains(&"BLOCK_NO"));
        assert!(headers.contains(&"TRIP_NO"));
    }

    #[test]
    fn test_gtfs_block_config() {
        let config = ExportPreset::GtfsBlock.to_config();

        let headers: Vec<&str> = config.columns.iter().map(|c| c.header.as_str()).collect();
        assert!(headers.contains(&"block_id"));
        assert!(headers.contains(&"trip_id"));
    }
}
