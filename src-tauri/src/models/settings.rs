//! App-wide settings model
//!
//! These settings apply to all projects and are stored in a JSON file
//! in the app data directory.

use serde::{Deserialize, Serialize};

/// App-wide settings stored outside of the database
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    /// Author's legal name (used in contact info on title pages)
    #[serde(default)]
    pub author_name: Option<String>,

    /// First line of contact address (e.g., street address)
    #[serde(default)]
    pub contact_address_line1: Option<String>,

    /// Second line of contact address (e.g., city, country, postal code)
    #[serde(default)]
    pub contact_address_line2: Option<String>,

    /// Phone number
    #[serde(default)]
    pub contact_phone: Option<String>,

    /// Email address
    #[serde(default)]
    pub contact_email: Option<String>,
}

impl AppSettings {
    /// Create new default settings
    pub fn new() -> Self {
        Self::default()
    }
}
