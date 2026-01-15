//! Transit agency model.

use serde::{Deserialize, Serialize};

/// A transit agency operating routes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agency {
    /// Unique identifier for the agency.
    pub id: Option<String>,

    /// Full name of the transit agency.
    pub name: String,

    /// URL of the transit agency's website.
    pub url: String,

    /// Timezone where the transit agency is located (e.g., "Europe/London").
    pub timezone: String,

    /// Primary language used by this transit agency.
    pub lang: Option<String>,

    /// Phone number for the agency.
    pub phone: Option<String>,

    /// URL of a page for purchasing tickets online.
    pub fare_url: Option<String>,

    /// Email address for customer service.
    pub email: Option<String>,
}

impl Agency {
    pub fn new(name: impl Into<String>, url: impl Into<String>, timezone: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            url: url.into(),
            timezone: timezone.into(),
            lang: None,
            phone: None,
            fare_url: None,
            email: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}
