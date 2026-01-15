//! Agency mapping from TXC Operators.

use super::MappingContext;
use crate::ConversionOptions;
use transit_core::{AdapterError, Agency};
use txc_parser::TxcDocument;

/// Map TXC operators to GTFS agencies.
pub fn map_agencies(
    doc: &TxcDocument,
    options: &ConversionOptions,
    _ctx: &mut MappingContext,
) -> Result<Vec<Agency>, AdapterError> {
    let mut agencies = Vec::new();

    for operator in &doc.operators {
        let name = operator.display_name().to_string();

        let agency = Agency {
            id: Some(operator.id.clone()),
            name,
            url: options.default_agency_url.clone(),
            timezone: options.default_timezone.clone(),
            lang: Some("en".to_string()),
            phone: None,
            fare_url: None,
            email: None,
        };

        agencies.push(agency);
    }

    // If no operators found, create a default agency
    if agencies.is_empty() {
        agencies.push(Agency {
            id: Some("DEFAULT".to_string()),
            name: "Unknown Operator".to_string(),
            url: options.default_agency_url.clone(),
            timezone: options.default_timezone.clone(),
            lang: Some("en".to_string()),
            phone: None,
            fare_url: None,
            email: None,
        });
    }

    Ok(agencies)
}
