//! TXC document reader.

use crate::schema::TxcSchemaVersion;
use crate::types::*;
use crate::TxcDocument;
use chrono::NaiveDate;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use transit_core::ParseError;

/// Options for reading TXC documents.
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    /// Whether to be lenient with malformed data.
    pub lenient: bool,
}

/// TXC document reader.
pub struct TxcReader;

impl TxcReader {
    /// Read a TXC document from a file path.
    pub fn read_path(path: &Path, options: ReadOptions) -> Result<TxcDocument, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut doc = Self::read_impl(reader, options)?;
        doc.filename = path.file_name().and_then(|s| s.to_str()).map(String::from);
        Ok(doc)
    }

    /// Read a TXC document from bytes.
    pub fn read_bytes(bytes: &[u8], options: ReadOptions) -> Result<TxcDocument, ParseError> {
        let reader = std::io::Cursor::new(bytes);
        Self::read_impl(reader, options)
    }

    /// Read a TXC document from a string.
    pub fn read_str(xml: &str, options: ReadOptions) -> Result<TxcDocument, ParseError> {
        Self::read_bytes(xml.as_bytes(), options)
    }

    fn read_impl<R: BufRead>(reader: R, options: ReadOptions) -> Result<TxcDocument, ParseError> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut doc = TxcDocument::new();
        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"TransXChange" => {
                            Self::parse_root_attrs(e, &mut doc)?;
                        }
                        b"Operators" => {
                            doc.operators =
                                Self::parse_operators(&mut xml_reader, &mut buf, &options)?;
                        }
                        b"StopPoints" => {
                            doc.stop_points =
                                Self::parse_stop_points(&mut xml_reader, &mut buf, &options)?;
                        }
                        b"RouteSections" => {
                            doc.route_sections =
                                Self::parse_route_sections(&mut xml_reader, &mut buf, &options)?;
                        }
                        b"Routes" => {
                            doc.routes = Self::parse_routes(&mut xml_reader, &mut buf, &options)?;
                        }
                        b"JourneyPatternSections" => {
                            doc.journey_pattern_sections = Self::parse_journey_pattern_sections(
                                &mut xml_reader,
                                &mut buf,
                                &options,
                            )?;
                        }
                        b"Services" => {
                            let (services, journey_patterns) =
                                Self::parse_services(&mut xml_reader, &mut buf, &options)?;
                            doc.services = services;
                            doc.journey_patterns = journey_patterns;
                        }
                        b"VehicleJourneys" => {
                            doc.vehicle_journeys =
                                Self::parse_vehicle_journeys(&mut xml_reader, &mut buf, &options)?;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParseError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(doc)
    }

    fn parse_root_attrs(e: &BytesStart, doc: &mut TxcDocument) -> Result<(), ParseError> {
        for attr in e.attributes().flatten() {
            match attr.key.as_ref() {
                b"SchemaVersion" => {
                    doc.schema_version = String::from_utf8_lossy(&attr.value).to_string();
                }
                b"CreationDateTime" => {
                    doc.creation_datetime = Some(String::from_utf8_lossy(&attr.value).to_string());
                }
                b"ModificationDateTime" => {
                    doc.modification_datetime =
                        Some(String::from_utf8_lossy(&attr.value).to_string());
                }
                b"xsi:schemaLocation" => {
                    let location = String::from_utf8_lossy(&attr.value);
                    let version = TxcSchemaVersion::from_schema_location(&location);
                    if doc.schema_version.is_empty() {
                        doc.schema_version = version.as_str().to_string();
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_operators<R: BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        _options: &ReadOptions,
    ) -> Result<Vec<TxcOperator>, ParseError> {
        let mut operators = Vec::new();
        let mut current: Option<TxcOperator> = None;
        let mut current_tag = String::new();

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"Operator" | b"LicensedOperator" => {
                            let id = Self::get_attr(e, b"id").unwrap_or_default();
                            current = Some(TxcOperator::new(id));
                        }
                        _ => {
                            current_tag = String::from_utf8_lossy(name.as_ref()).to_string();
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref mut op) = current {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_tag.as_str() {
                            "NationalOperatorCode" => op.national_operator_code = Some(text),
                            "OperatorShortName" => op.operator_short_name = Some(text),
                            "OperatorNameOnLicence" => op.operator_name_on_licence = Some(text),
                            "TradingName" => op.trading_name = Some(text),
                            "LicenceNumber" => op.licence_number = Some(text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    if name.as_ref() == b"Operator" || name.as_ref() == b"LicensedOperator" {
                        if let Some(op) = current.take() {
                            operators.push(op);
                        }
                    } else if name.as_ref() == b"Operators" {
                        break;
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParseError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(operators)
    }

    fn parse_stop_points<R: BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        _options: &ReadOptions,
    ) -> Result<Vec<TxcStopPoint>, ParseError> {
        let mut stops = Vec::new();
        let mut current: Option<TxcStopPoint> = None;
        let mut current_tag = String::new();
        let mut in_location = false;

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"StopPoint" | b"AnnotatedStopPointRef" => {
                            current = Some(TxcStopPoint::new(""));
                        }
                        b"Location" => {
                            in_location = true;
                        }
                        _ => {
                            current_tag = String::from_utf8_lossy(name.as_ref()).to_string();
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref mut stop) = current {
                        let text = e.unescape().unwrap_or_default().to_string();
                        if in_location {
                            match current_tag.as_str() {
                                "Latitude" => stop.latitude = text.parse().ok(),
                                "Longitude" => stop.longitude = text.parse().ok(),
                                "Easting" => stop.easting = text.parse().ok(),
                                "Northing" => stop.northing = text.parse().ok(),
                                _ => {}
                            }
                        } else {
                            match current_tag.as_str() {
                                "AtcoCode" | "StopPointRef" => stop.atco_code = text,
                                "NaptanStopType" => stop.naptan_stop_type = Some(text),
                                "CommonName" => stop.common_name = Some(text),
                                "ShortCommonName" => stop.short_common_name = Some(text),
                                "Landmark" => stop.landmark = Some(text),
                                "Street" => stop.street = Some(text),
                                "Indicator" => stop.indicator = Some(text),
                                "Bearing" => stop.bearing = Some(text),
                                "LocalityName" => stop.locality_name = Some(text),
                                "LocalityQualifier" => stop.locality_qualifier = Some(text),
                                "ParentLocalityRef" => stop.parent_locality_ref = Some(text),
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"StopPoint" | b"AnnotatedStopPointRef" => {
                            if let Some(stop) = current.take() {
                                if !stop.atco_code.is_empty() {
                                    stops.push(stop);
                                }
                            }
                        }
                        b"Location" => {
                            in_location = false;
                        }
                        b"StopPoints" => break,
                        _ => {}
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParseError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(stops)
    }

    fn parse_route_sections<R: BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        _options: &ReadOptions,
    ) -> Result<Vec<TxcRouteSection>, ParseError> {
        let mut sections = Vec::new();
        let mut current_section: Option<TxcRouteSection> = None;
        let mut current_link: Option<TxcRouteLink> = None;
        let mut current_tag = String::new();

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"RouteSection" => {
                            let id = Self::get_attr(e, b"id").unwrap_or_default();
                            current_section = Some(TxcRouteSection {
                                id,
                                route_links: Vec::new(),
                            });
                        }
                        b"RouteLink" => {
                            let id = Self::get_attr(e, b"id").unwrap_or_default();
                            current_link = Some(TxcRouteLink {
                                id,
                                from_stop_ref: String::new(),
                                to_stop_ref: String::new(),
                                direction: None,
                                distance: None,
                                track: None,
                            });
                        }
                        _ => {
                            current_tag = String::from_utf8_lossy(name.as_ref()).to_string();
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref mut link) = current_link {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_tag.as_str() {
                            "From" | "StopPointRef" if link.from_stop_ref.is_empty() => {
                                link.from_stop_ref = text;
                            }
                            "To" => {
                                link.to_stop_ref = text;
                            }
                            "Direction" => {
                                link.direction = Some(text);
                            }
                            "Distance" => {
                                link.distance = text.parse().ok();
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"RouteLink" => {
                            if let (Some(ref mut section), Some(link)) =
                                (&mut current_section, current_link.take())
                            {
                                section.route_links.push(link);
                            }
                        }
                        b"RouteSection" => {
                            if let Some(section) = current_section.take() {
                                sections.push(section);
                            }
                        }
                        b"RouteSections" => break,
                        _ => {}
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParseError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(sections)
    }

    fn parse_routes<R: BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        _options: &ReadOptions,
    ) -> Result<Vec<TxcRoute>, ParseError> {
        let mut routes = Vec::new();
        let mut current: Option<TxcRoute> = None;
        let mut current_tag = String::new();

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"Route" => {
                            let id = Self::get_attr(e, b"id").unwrap_or_default();
                            current = Some(TxcRoute {
                                id,
                                private_code: None,
                                description: None,
                                route_section_refs: Vec::new(),
                            });
                        }
                        _ => {
                            current_tag = String::from_utf8_lossy(name.as_ref()).to_string();
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref mut route) = current {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_tag.as_str() {
                            "PrivateCode" => route.private_code = Some(text),
                            "Description" => route.description = Some(text),
                            "RouteSectionRef" => route.route_section_refs.push(text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"Route" => {
                            if let Some(route) = current.take() {
                                routes.push(route);
                            }
                        }
                        b"Routes" => break,
                        _ => {}
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParseError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(routes)
    }

    fn parse_journey_pattern_sections<R: BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        _options: &ReadOptions,
    ) -> Result<Vec<TxcJourneyPatternSection>, ParseError> {
        let mut sections = Vec::new();
        let mut current_section: Option<TxcJourneyPatternSection> = None;
        let mut current_link: Option<TxcJourneyPatternTimingLink> = None;
        let mut current_from: Option<TxcJourneyPatternStopUsage> = None;
        let mut current_to: Option<TxcJourneyPatternStopUsage> = None;
        let mut current_tag = String::new();
        let mut in_from = false;
        let mut in_to = false;

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"JourneyPatternSection" => {
                            let id = Self::get_attr(e, b"id").unwrap_or_default();
                            current_section = Some(TxcJourneyPatternSection {
                                id,
                                timing_links: Vec::new(),
                            });
                        }
                        b"JourneyPatternTimingLink" => {
                            let id = Self::get_attr(e, b"id").unwrap_or_default();
                            current_link = Some(TxcJourneyPatternTimingLink {
                                id,
                                from: TxcJourneyPatternStopUsage::new(""),
                                to: TxcJourneyPatternStopUsage::new(""),
                                route_link_ref: None,
                                run_time: None,
                            });
                        }
                        b"From" => {
                            in_from = true;
                            current_from = Some(TxcJourneyPatternStopUsage::new(""));
                        }
                        b"To" => {
                            in_to = true;
                            current_to = Some(TxcJourneyPatternStopUsage::new(""));
                        }
                        _ => {
                            current_tag = String::from_utf8_lossy(name.as_ref()).to_string();
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if in_from {
                        if let Some(ref mut from) = current_from {
                            match current_tag.as_str() {
                                "StopPointRef" => from.stop_point_ref = text,
                                "WaitTime" => from.wait_time = Some(text),
                                _ => {}
                            }
                        }
                    } else if in_to {
                        if let Some(ref mut to) = current_to {
                            match current_tag.as_str() {
                                "StopPointRef" => to.stop_point_ref = text,
                                "WaitTime" => to.wait_time = Some(text),
                                _ => {}
                            }
                        }
                    } else if let Some(ref mut link) = current_link {
                        match current_tag.as_str() {
                            "RouteLinkRef" => link.route_link_ref = Some(text),
                            "RunTime" => link.run_time = Some(text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"From" => {
                            in_from = false;
                            if let (Some(ref mut link), Some(from)) =
                                (&mut current_link, current_from.take())
                            {
                                link.from = from;
                            }
                        }
                        b"To" => {
                            in_to = false;
                            if let (Some(ref mut link), Some(to)) =
                                (&mut current_link, current_to.take())
                            {
                                link.to = to;
                            }
                        }
                        b"JourneyPatternTimingLink" => {
                            if let (Some(ref mut section), Some(link)) =
                                (&mut current_section, current_link.take())
                            {
                                section.timing_links.push(link);
                            }
                        }
                        b"JourneyPatternSection" => {
                            if let Some(section) = current_section.take() {
                                sections.push(section);
                            }
                        }
                        b"JourneyPatternSections" => break,
                        _ => {}
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParseError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(sections)
    }

    fn parse_services<R: BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        _options: &ReadOptions,
    ) -> Result<(Vec<TxcService>, Vec<TxcJourneyPattern>), ParseError> {
        let mut services = Vec::new();
        let mut journey_patterns = Vec::new();
        let mut current: Option<TxcService> = None;
        let mut current_line: Option<TxcLine> = None;
        let mut current_jp: Option<TxcJourneyPattern> = None;
        let mut current_tag = String::new();
        let mut in_line = false;
        let mut in_operating_period = false;
        let mut _in_operating_profile = false;
        let mut _in_regular_day_type = false;
        let mut in_days_of_week = false;
        let mut in_journey_pattern = false;
        let mut temp_start_date: Option<String> = None;
        let mut temp_end_date: Option<String> = None;
        let mut temp_days = TxcDaysOfWeek::default();

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"Service" => {
                            current = Some(TxcService::new(""));
                        }
                        b"Line" => {
                            let id = Self::get_attr(e, b"id").unwrap_or_default();
                            current_line = Some(TxcLine {
                                id,
                                line_name: String::new(),
                                outbound_description: None,
                                inbound_description: None,
                            });
                            in_line = true;
                        }
                        b"OperatingPeriod" => {
                            in_operating_period = true;
                        }
                        b"OperatingProfile" => {
                            _in_operating_profile = true;
                        }
                        b"RegularDayType" => {
                            _in_regular_day_type = true;
                        }
                        b"DaysOfWeek" => {
                            in_days_of_week = true;
                            temp_days = TxcDaysOfWeek::default();
                        }
                        b"JourneyPattern" => {
                            let id = Self::get_attr(e, b"id").unwrap_or_default();
                            current_jp = Some(TxcJourneyPattern {
                                id,
                                destination_display: None,
                                direction: None,
                                description: None,
                                route_ref: None,
                                section_refs: Vec::new(),
                            });
                            in_journey_pattern = true;
                        }
                        _ => {
                            current_tag = String::from_utf8_lossy(name.as_ref()).to_string();
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    if in_days_of_week {
                        let name = e.name();
                        match name.as_ref() {
                            b"Monday" => temp_days.monday = true,
                            b"Tuesday" => temp_days.tuesday = true,
                            b"Wednesday" => temp_days.wednesday = true,
                            b"Thursday" => temp_days.thursday = true,
                            b"Friday" => temp_days.friday = true,
                            b"Saturday" => temp_days.saturday = true,
                            b"Sunday" => temp_days.sunday = true,
                            b"MondayToFriday" => temp_days.monday_to_friday = true,
                            b"MondayToSaturday" => temp_days.monday_to_saturday = true,
                            b"MondayToSunday" => temp_days.monday_to_sunday = true,
                            b"Weekend" => temp_days.weekend = true,
                            _ => {}
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();

                    if in_journey_pattern {
                        if let Some(ref mut jp) = current_jp {
                            match current_tag.as_str() {
                                "DestinationDisplay" => jp.destination_display = Some(text),
                                "Direction" => jp.direction = Some(text),
                                "Description" => jp.description = Some(text),
                                "RouteRef" => jp.route_ref = Some(text),
                                "JourneyPatternSectionRefs" => jp.section_refs.push(text),
                                _ => {}
                            }
                        }
                    } else if in_line {
                        if let Some(ref mut line) = current_line {
                            match current_tag.as_str() {
                                "LineName" => line.line_name = text,
                                "OutboundDescription" => line.outbound_description = Some(text),
                                "InboundDescription" => line.inbound_description = Some(text),
                                _ => {}
                            }
                        }
                    } else if in_operating_period {
                        match current_tag.as_str() {
                            "StartDate" => temp_start_date = Some(text),
                            "EndDate" => temp_end_date = Some(text),
                            _ => {}
                        }
                    } else if let Some(ref mut svc) = current {
                        match current_tag.as_str() {
                            "ServiceCode" => svc.service_code = text,
                            "PrivateCode" => svc.private_code = Some(text),
                            "RegisteredOperatorRef" => svc.registered_operator_ref = Some(text),
                            "Description" => svc.description = Some(text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"Line" => {
                            in_line = false;
                            if let (Some(ref mut svc), Some(line)) =
                                (&mut current, current_line.take())
                            {
                                svc.lines.push(line);
                            }
                        }
                        b"OperatingPeriod" => {
                            in_operating_period = false;
                            if let Some(ref mut svc) = current {
                                if let Some(start_str) = temp_start_date.take() {
                                    if let Ok(start) =
                                        NaiveDate::parse_from_str(&start_str, "%Y-%m-%d")
                                    {
                                        let end = temp_end_date.take().and_then(|s| {
                                            NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()
                                        });
                                        svc.operating_period = Some(TxcOperatingPeriod {
                                            start_date: start,
                                            end_date: end,
                                        });
                                    }
                                }
                            }
                        }
                        b"DaysOfWeek" => {
                            in_days_of_week = false;
                        }
                        b"RegularDayType" => {
                            _in_regular_day_type = false;
                            if let Some(ref mut svc) = current {
                                let profile =
                                    svc.operating_profile.get_or_insert_with(Default::default);
                                profile.regular_day_type = Some(TxcRegularDayType {
                                    days_of_week: Some(temp_days.clone()),
                                    holidays_only: false,
                                });
                            }
                        }
                        b"OperatingProfile" => {
                            _in_operating_profile = false;
                        }
                        b"JourneyPattern" => {
                            in_journey_pattern = false;
                            if let Some(jp) = current_jp.take() {
                                journey_patterns.push(jp);
                            }
                        }
                        b"Service" => {
                            if let Some(svc) = current.take() {
                                services.push(svc);
                            }
                        }
                        b"Services" => break,
                        _ => {}
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParseError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok((services, journey_patterns))
    }

    fn parse_vehicle_journeys<R: BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        _options: &ReadOptions,
    ) -> Result<Vec<TxcVehicleJourney>, ParseError> {
        let mut journeys = Vec::new();
        let mut current: Option<TxcVehicleJourney> = None;
        let mut current_tag = String::new();

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"VehicleJourney" => {
                            current = Some(TxcVehicleJourney::new("", "", "", ""));
                        }
                        _ => {
                            current_tag = String::from_utf8_lossy(name.as_ref()).to_string();
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref mut vj) = current {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_tag.as_str() {
                            "PrivateCode" => vj.private_code = Some(text),
                            "VehicleJourneyCode" => vj.vehicle_journey_code = text,
                            "ServiceRef" => vj.service_ref = text,
                            "LineRef" => vj.line_ref = text,
                            "JourneyPatternRef" => vj.journey_pattern_ref = Some(text),
                            "DepartureTime" => vj.departure_time = text,
                            "DestinationDisplay" => vj.destination_display = Some(text),
                            "Direction" => vj.direction = Some(text),
                            "BlockRef" => vj.block_ref = Some(text),
                            "Note" => vj.note = Some(text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"VehicleJourney" => {
                            if let Some(vj) = current.take() {
                                journeys.push(vj);
                            }
                        }
                        b"VehicleJourneys" => break,
                        _ => {}
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParseError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(journeys)
    }

    fn get_attr(e: &BytesStart, name: &[u8]) -> Option<String> {
        e.attributes()
            .flatten()
            .find(|a| a.key.as_ref() == name)
            .map(|a| String::from_utf8_lossy(&a.value).to_string())
    }
}
