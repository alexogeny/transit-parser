//! Trip mapping from TXC VehicleJourneys.

use super::MappingContext;
use transit_core::{AdapterError, DirectionId, Trip};
use txc_parser::TxcDocument;

/// Map TXC vehicle journeys to GTFS trips.
pub fn map_trips(
    doc: &TxcDocument,
    ctx: &mut MappingContext,
) -> Result<Vec<Trip>, AdapterError> {
    let mut trips = Vec::new();

    for vj in &doc.vehicle_journeys {
        let trip_id = ctx.next_trip_id();

        // Record mapping
        ctx.trip_mapping
            .insert(vj.vehicle_journey_code.clone(), trip_id.clone());

        // Look up route ID from line reference
        let route_id = ctx
            .route_mapping
            .get(&vj.line_ref)
            .cloned()
            .unwrap_or_else(|| vj.line_ref.clone());

        // Look up service ID
        let service_id = ctx
            .service_mapping
            .get(&vj.service_ref)
            .cloned()
            .unwrap_or_else(|| vj.service_ref.clone());

        // Map direction
        let direction_id = vj.direction.as_ref().and_then(|d| {
            match d.to_lowercase().as_str() {
                "outbound" | "out" => Some(DirectionId::Outbound),
                "inbound" | "in" => Some(DirectionId::Inbound),
                _ => None,
            }
        });

        let trip = Trip {
            id: trip_id,
            route_id,
            service_id,
            headsign: vj.destination_display.clone(),
            short_name: None,
            direction_id,
            block_id: vj.block_ref.clone(),
            shape_id: None, // Will be set in shapes mapping if enabled
            wheelchair_accessible: None,
            bikes_allowed: None,
            vehicle_journey_code: Some(vj.vehicle_journey_code.clone()),
        };

        trips.push(trip);
    }

    Ok(trips)
}
