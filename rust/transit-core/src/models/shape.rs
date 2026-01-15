//! Shape model for geographic paths.

use serde::{Deserialize, Serialize};

/// A geographic shape representing a route's path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Shape {
    /// Unique identifier for the shape.
    pub id: String,

    /// Ordered list of points making up the shape.
    pub points: Vec<ShapePoint>,
}

impl Shape {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            points: Vec::new(),
        }
    }

    pub fn with_points(mut self, points: Vec<ShapePoint>) -> Self {
        self.points = points;
        self
    }

    pub fn add_point(&mut self, point: ShapePoint) {
        self.points.push(point);
    }
}

/// A point in a shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShapePoint {
    /// Latitude (WGS 84).
    pub latitude: f64,

    /// Longitude (WGS 84).
    pub longitude: f64,

    /// Sequence order of this point.
    pub sequence: u32,

    /// Distance traveled from first point (meters).
    pub dist_traveled: Option<f64>,
}

impl ShapePoint {
    pub fn new(latitude: f64, longitude: f64, sequence: u32) -> Self {
        Self {
            latitude,
            longitude,
            sequence,
            dist_traveled: None,
        }
    }

    pub fn with_distance(mut self, distance: f64) -> Self {
        self.dist_traveled = Some(distance);
        self
    }
}
