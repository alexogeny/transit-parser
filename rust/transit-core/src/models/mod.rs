//! Core transit data models.
//!
//! These models represent transit data in a unified, format-agnostic way.
//! They can be populated from TXC, GTFS, or other transit data formats.

mod agency;
mod calendar;
mod route;
mod shape;
mod stop;
mod stop_time;
mod trip;

pub use agency::Agency;
pub use calendar::{Calendar, CalendarDate, ExceptionType, ServiceAvailability};
pub use route::{Route, RouteType};
pub use shape::{Shape, ShapePoint};
pub use stop::{LocationType, Stop, WheelchairBoarding};
pub use stop_time::{PickupDropoffType, StopTime, Timepoint};
pub use trip::{BikesAllowed, DirectionId, Trip, WheelchairAccessible};

/// A complete transit feed containing all entities.
#[derive(Debug, Clone, Default)]
pub struct TransitFeed {
    pub agencies: Vec<Agency>,
    pub stops: Vec<Stop>,
    pub routes: Vec<Route>,
    pub trips: Vec<Trip>,
    pub stop_times: Vec<StopTime>,
    pub calendars: Vec<Calendar>,
    pub calendar_dates: Vec<CalendarDate>,
    pub shapes: Vec<Shape>,
}

impl TransitFeed {
    pub fn new() -> Self {
        Self::default()
    }
}
