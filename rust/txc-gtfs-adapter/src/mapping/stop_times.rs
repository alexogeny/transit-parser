//! Stop times generation from TXC JourneyPatterns.

use super::{parse_duration, parse_time_to_seconds, MappingContext};
use std::collections::HashMap;
use transit_core::{AdapterError, PickupDropoffType, StopTime, Timepoint, Warning};
use txc_parser::{StopActivity, TxcDocument, TxcJourneyPattern, TxcJourneyPatternSection};

/// Generate GTFS stop times from TXC journey patterns and vehicle journeys.
pub fn map_stop_times(
    doc: &TxcDocument,
    ctx: &mut MappingContext,
    warnings: &mut Vec<Warning>,
) -> Result<Vec<StopTime>, AdapterError> {
    let mut stop_times = Vec::new();

    // Build journey pattern sections lookup (JPS id -> section)
    let jp_sections: HashMap<String, &TxcJourneyPatternSection> = doc
        .journey_pattern_sections
        .iter()
        .map(|s| (s.id.clone(), s))
        .collect();

    // Build journey patterns lookup (JP id -> pattern with section refs)
    let jp_lookup: HashMap<String, &TxcJourneyPattern> = doc
        .journey_patterns
        .iter()
        .map(|jp| (jp.id.clone(), jp))
        .collect();

    // Process each vehicle journey
    for vj in &doc.vehicle_journeys {
        // Get the trip ID
        let trip_id = match ctx.trip_mapping.get(&vj.vehicle_journey_code) {
            Some(id) => id.clone(),
            None => {
                warnings.push(Warning::new(
                    "MISSING_TRIP",
                    format!(
                        "No trip found for vehicle journey: {}",
                        vj.vehicle_journey_code
                    ),
                ));
                continue;
            }
        };

        // Get departure time in seconds
        let departure_seconds = match parse_time_to_seconds(&vj.departure_time) {
            Some(s) => s,
            None => {
                warnings.push(Warning::new(
                    "INVALID_TIME",
                    format!(
                        "Invalid departure time '{}' for journey {}",
                        vj.departure_time, vj.vehicle_journey_code
                    ),
                ));
                continue;
            }
        };

        // Find the journey pattern reference
        let jp_ref = match &vj.journey_pattern_ref {
            Some(r) => r,
            None => {
                warnings.push(Warning::new(
                    "MISSING_JP_REF",
                    format!(
                        "No journey pattern reference for vehicle journey: {}",
                        vj.vehicle_journey_code
                    ),
                ));
                continue;
            }
        };

        // Look up the journey pattern to get its section refs
        let journey_pattern = match jp_lookup.get(jp_ref) {
            Some(jp) => jp,
            None => {
                warnings.push(Warning::new(
                    "MISSING_JP",
                    format!("Journey pattern not found: {}", jp_ref),
                ));
                continue;
            }
        };

        // Generate stop times from all sections referenced by this journey pattern
        let mut current_time = departure_seconds;
        let mut sequence = 0u32;

        for section_ref in &journey_pattern.section_refs {
            let section = match jp_sections.get(section_ref) {
                Some(s) => s,
                None => {
                    warnings.push(Warning::new(
                        "MISSING_SECTION",
                        format!("Journey pattern section not found: {}", section_ref),
                    ));
                    continue;
                }
            };

            // Generate stop times from this section
            let (times, new_time, new_seq) = generate_stop_times_from_section(
                &trip_id,
                section,
                current_time,
                sequence,
                ctx,
                warnings,
            );
            stop_times.extend(times);
            current_time = new_time;
            sequence = new_seq;
        }
    }

    Ok(stop_times)
}

/// Generate stop times from a journey pattern section.
/// Returns (stop_times, final_time, final_sequence) to allow chaining multiple sections.
fn generate_stop_times_from_section(
    trip_id: &str,
    section: &TxcJourneyPatternSection,
    start_time: u32,
    start_sequence: u32,
    ctx: &MappingContext,
    warnings: &mut Vec<Warning>,
) -> (Vec<StopTime>, u32, u32) {
    let mut stop_times = Vec::new();
    let mut current_time = start_time;
    let mut sequence = start_sequence;

    for (i, link) in section.timing_links.iter().enumerate() {
        // First stop (from) - only add if this is the first link in the section
        // and we haven't already added stops (sequence == start_sequence)
        if i == 0 && sequence == start_sequence {
            let stop_id = ctx
                .stop_mapping
                .get(&link.from.stop_point_ref)
                .cloned()
                .unwrap_or_else(|| link.from.stop_point_ref.clone());

            let (pickup_type, drop_off_type) = activity_to_pickup_dropoff(link.from.activity);

            stop_times.push(StopTime {
                trip_id: trip_id.to_string(),
                arrival_time: Some(current_time),
                departure_time: Some(current_time),
                stop_id,
                stop_sequence: sequence,
                stop_headsign: None,
                pickup_type,
                drop_off_type,
                continuous_pickup: None,
                continuous_drop_off: None,
                shape_dist_traveled: None,
                timepoint: timing_status_to_timepoint(link.from.timing_status),
            });
            sequence += 1;

            // Add wait time at first stop if specified
            if let Some(ref wait) = link.from.wait_time {
                if let Some(wait_secs) = parse_duration(wait) {
                    current_time += wait_secs;
                }
            }
        }

        // Add run time
        if let Some(ref run_time) = link.run_time {
            if let Some(run_secs) = parse_duration(run_time) {
                current_time += run_secs;
            }
        } else {
            // Default to 2 minutes if no run time specified
            current_time += 120;
            warnings.push(Warning::new(
                "DEFAULT_RUN_TIME",
                format!("No run time for link {}, using default 2 minutes", link.id),
            ));
        }

        // To stop
        let stop_id = ctx
            .stop_mapping
            .get(&link.to.stop_point_ref)
            .cloned()
            .unwrap_or_else(|| link.to.stop_point_ref.clone());

        let (pickup_type, drop_off_type) = activity_to_pickup_dropoff(link.to.activity);

        // Add wait time at to stop
        let mut departure_time = current_time;
        if let Some(ref wait) = link.to.wait_time {
            if let Some(wait_secs) = parse_duration(wait) {
                departure_time += wait_secs;
            }
        }

        stop_times.push(StopTime {
            trip_id: trip_id.to_string(),
            arrival_time: Some(current_time),
            departure_time: Some(departure_time),
            stop_id,
            stop_sequence: sequence,
            stop_headsign: None,
            pickup_type,
            drop_off_type,
            continuous_pickup: None,
            continuous_drop_off: None,
            shape_dist_traveled: None,
            timepoint: timing_status_to_timepoint(link.to.timing_status),
        });

        current_time = departure_time;
        sequence += 1;
    }

    (stop_times, current_time, sequence)
}

/// Convert TXC stop activity to GTFS pickup/dropoff types.
fn activity_to_pickup_dropoff(
    activity: Option<StopActivity>,
) -> (PickupDropoffType, PickupDropoffType) {
    match activity {
        Some(StopActivity::PickUp) => (PickupDropoffType::Regular, PickupDropoffType::None),
        Some(StopActivity::SetDown) => (PickupDropoffType::None, PickupDropoffType::Regular),
        Some(StopActivity::PickUpAndSetDown) | None => {
            (PickupDropoffType::Regular, PickupDropoffType::Regular)
        }
        Some(StopActivity::Pass) => (PickupDropoffType::None, PickupDropoffType::None),
    }
}

/// Convert TXC timing status to GTFS timepoint.
fn timing_status_to_timepoint(status: Option<txc_parser::TimingStatus>) -> Timepoint {
    match status {
        Some(txc_parser::TimingStatus::PrincipalTimingPoint)
        | Some(txc_parser::TimingStatus::TimingPoint) => Timepoint::Exact,
        _ => Timepoint::Approximate,
    }
}
