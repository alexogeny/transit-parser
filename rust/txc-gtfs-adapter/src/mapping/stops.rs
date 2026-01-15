//! Stop mapping from TXC StopPoints.

use super::MappingContext;
use transit_core::{AdapterError, LocationType, Stop};
use txc_parser::TxcDocument;

/// Map TXC stop points to GTFS stops.
pub fn map_stops(
    doc: &TxcDocument,
    ctx: &mut MappingContext,
) -> Result<Vec<Stop>, AdapterError> {
    let mut stops = Vec::new();

    for txc_stop in &doc.stop_points {
        // Use ATCO code as stop_id
        let stop_id = txc_stop.atco_code.clone();

        // Record mapping
        ctx.stop_mapping
            .insert(txc_stop.atco_code.clone(), stop_id.clone());

        // Get coordinates - prefer lat/lon, fall back to converting easting/northing
        let (lat, lon) = get_coordinates(txc_stop);

        let stop = Stop {
            id: stop_id,
            code: Some(txc_stop.atco_code.clone()),
            name: txc_stop.display_name(),
            description: txc_stop.street.clone(),
            latitude: lat,
            longitude: lon,
            zone_id: None,
            url: None,
            location_type: LocationType::Stop,
            parent_station: None,
            timezone: None,
            wheelchair_boarding: None,
            platform_code: txc_stop.indicator.clone(),
            naptan_code: Some(txc_stop.atco_code.clone()),
            atco_code: Some(txc_stop.atco_code.clone()),
        };

        stops.push(stop);
    }

    Ok(stops)
}

/// Get coordinates from a TXC stop point.
fn get_coordinates(stop: &txc_parser::TxcStopPoint) -> (f64, f64) {
    // Prefer lat/lon if available
    if let (Some(lat), Some(lon)) = (stop.latitude, stop.longitude) {
        return (lat, lon);
    }

    // Convert easting/northing to lat/lon if available
    if let (Some(easting), Some(northing)) = (stop.easting, stop.northing) {
        return convert_osgb_to_wgs84(easting, northing);
    }

    // Default to 0,0 (will need to be enriched from NaPTAN)
    (0.0, 0.0)
}

/// Convert OSGB36 (British National Grid) coordinates to WGS84 lat/lon.
/// This is a simplified conversion - for production use, consider a proper geodetic library.
fn convert_osgb_to_wgs84(easting: i32, northing: i32) -> (f64, f64) {
    // Simplified Helmert transformation
    // For production, use proj or similar library

    let e = easting as f64;
    let n = northing as f64;

    // OSGB36 ellipsoid parameters
    let a = 6377563.396; // Semi-major axis
    let b = 6356256.909; // Semi-minor axis
    let f0 = 0.9996012717; // Central meridian scale factor
    let lat0 = 49.0_f64.to_radians(); // True origin latitude
    let lon0 = (-2.0_f64).to_radians(); // True origin longitude
    let n0 = -100000.0; // Northing of true origin
    let e0 = 400000.0; // Easting of true origin

    let e2 = (a * a - b * b) / (a * a);
    let n_val: f64 = (a - b) / (a + b);

    let mut phi = lat0;

    // Iterative calculation of latitude
    for _ in 0..10 {
        let m = b * f0
            * ((1.0 + n_val + 1.25 * n_val * n_val + 1.25 * n_val.powi(3)) * (phi - lat0)
                - (3.0 * n_val + 3.0 * n_val * n_val + 2.625 * n_val.powi(3))
                    * (phi - lat0).sin()
                    * (phi + lat0).cos()
                + (1.875 * n_val * n_val + 1.875 * n_val.powi(3))
                    * (2.0 * (phi - lat0)).sin()
                    * (2.0 * (phi + lat0)).cos()
                - 35.0 / 24.0
                    * n_val.powi(3)
                    * (3.0 * (phi - lat0)).sin()
                    * (3.0 * (phi + lat0)).cos());

        phi = (n - n0 - m) / (a * f0) + phi;
    }

    let sin_phi = phi.sin();
    let cos_phi = phi.cos();
    let nu = a * f0 / (1.0 - e2 * sin_phi * sin_phi).sqrt();
    let rho = a * f0 * (1.0 - e2) / (1.0 - e2 * sin_phi * sin_phi).powf(1.5);
    let eta2 = nu / rho - 1.0;

    let tan_phi = phi.tan();
    let sec_phi = 1.0 / cos_phi;

    let vii = tan_phi / (2.0 * rho * nu);
    let viii = tan_phi / (24.0 * rho * nu.powi(3))
        * (5.0 + 3.0 * tan_phi * tan_phi + eta2 - 9.0 * tan_phi * tan_phi * eta2);
    let ix = tan_phi / (720.0 * rho * nu.powi(5))
        * (61.0 + 90.0 * tan_phi * tan_phi + 45.0 * tan_phi.powi(4));
    let x = sec_phi / nu;
    let xi = sec_phi / (6.0 * nu.powi(3)) * (nu / rho + 2.0 * tan_phi * tan_phi);
    let xii = sec_phi / (120.0 * nu.powi(5))
        * (5.0 + 28.0 * tan_phi * tan_phi + 24.0 * tan_phi.powi(4));
    let xiia = sec_phi / (5040.0 * nu.powi(7))
        * (61.0 + 662.0 * tan_phi * tan_phi + 1320.0 * tan_phi.powi(4) + 720.0 * tan_phi.powi(6));

    let de = e - e0;

    let lat = phi - vii * de * de + viii * de.powi(4) - ix * de.powi(6);
    let lon = lon0 + x * de - xi * de.powi(3) + xii * de.powi(5) - xiia * de.powi(7);

    // Convert to degrees
    (lat.to_degrees(), lon.to_degrees())
}
