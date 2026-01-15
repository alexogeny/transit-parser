//! Route mapping from TXC Services/Lines.

use super::MappingContext;
use transit_core::{AdapterError, Route, RouteType};
use txc_parser::TxcDocument;

/// Map TXC services/lines to GTFS routes.
pub fn map_routes(
    doc: &TxcDocument,
    ctx: &mut MappingContext,
) -> Result<Vec<Route>, AdapterError> {
    let mut routes = Vec::new();

    for service in &doc.services {
        let agency_id = service.registered_operator_ref.clone();

        for line in &service.lines {
            let route_id = format!("{}_{}", service.service_code, line.id);

            // Record mapping
            ctx.route_mapping.insert(line.id.clone(), route_id.clone());

            let route = Route {
                id: route_id,
                agency_id: agency_id.clone(),
                short_name: Some(line.line_name.clone()),
                long_name: service.description.clone().or_else(|| {
                    // Construct long name from outbound/inbound descriptions
                    match (&line.outbound_description, &line.inbound_description) {
                        (Some(out), Some(inb)) => Some(format!("{} - {}", out, inb)),
                        (Some(out), None) => Some(out.clone()),
                        (None, Some(inb)) => Some(inb.clone()),
                        (None, None) => None,
                    }
                }),
                description: None,
                route_type: RouteType::Bus, // TXC is primarily for bus data
                url: None,
                color: None,
                text_color: None,
                sort_order: None,
                continuous_pickup: None,
                continuous_drop_off: None,
                network_id: None,
            };

            routes.push(route);
        }
    }

    Ok(routes)
}
