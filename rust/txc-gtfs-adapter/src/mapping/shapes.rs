//! Shape generation from TXC RouteSections.

use super::MappingContext;
use transit_core::{AdapterError, Shape, ShapePoint};
use txc_parser::TxcDocument;

/// Generate GTFS shapes from TXC route sections.
pub fn map_shapes(
    doc: &TxcDocument,
    ctx: &mut MappingContext,
) -> Result<Vec<Shape>, AdapterError> {
    let mut shapes = Vec::new();

    for route_section in &doc.route_sections {
        let mut points = Vec::new();
        let mut sequence = 0u32;
        let mut total_distance = 0.0f64;

        for link in &route_section.route_links {
            // Add track points if available
            if let Some(ref track) = link.track {
                for track_point in track {
                    points.push(ShapePoint {
                        latitude: track_point.latitude,
                        longitude: track_point.longitude,
                        sequence,
                        dist_traveled: Some(total_distance),
                    });
                    sequence += 1;
                }
            }

            // Update distance
            if let Some(dist) = link.distance {
                total_distance += dist as f64;
            }
        }

        // Only create shape if we have points
        if !points.is_empty() {
            let shape_id = ctx.next_shape_id();
            shapes.push(Shape {
                id: shape_id,
                points,
            });
        }
    }

    Ok(shapes)
}
