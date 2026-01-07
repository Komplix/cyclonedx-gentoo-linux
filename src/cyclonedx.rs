use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct Bom {
    #[serde(rename = "bomFormat")]
    pub bom_format: String,
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    pub version: u32,
    pub metadata: Metadata,
    pub components: Vec<Component>,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub timestamp: String,
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<Component>,
}

#[derive(Debug, Serialize)]
pub struct Tool {
    pub vendor: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct Component {
    #[serde(rename = "type")]
    pub component_type: String,
    pub group: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub licenses: Vec<LicenseChoice>,
    pub purl: String,
}

#[derive(Debug, Serialize)]
pub struct LicenseChoice {
    pub license: License,
}

#[derive(Debug, Serialize)]
pub struct License {
    pub name: String,
}

impl Bom {
    pub fn new() -> Self {
        let now: DateTime<Utc> = Utc::now();
        Bom {
            bom_format: "CycloneDX".to_string(),
            spec_version: "1.5".to_string(),
            serial_number: format!("urn:uuid:{}", Uuid::new_v4()),
            version: 1,
            metadata: Metadata {
                timestamp: now.to_rfc3339(),
                tools: vec![Tool {
                    vendor: "cyclonedx-gentoo".to_string(),
                    name: "cyclonedx-gentoo".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                }],
                component: None,
            },
            components: Vec::new(),
        }
    }
}
