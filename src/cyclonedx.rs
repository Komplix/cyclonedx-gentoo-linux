//! CycloneDX SBOM models and implementation.


use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents the top-level CycloneDX Bill of Materials (BOM) structure.
#[derive(Debug, Serialize)]
pub struct Bom {
    /// The format of the BOM, usually "CycloneDX".
    #[serde(rename = "bomFormat")]
    pub bom_format: String,
    /// The version of the CycloneDX specification.
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    /// A unique serial number for the BOM.
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    /// The version of this specific BOM instance.
    pub version: u32,
    /// Metadata about the BOM, including creation time and tools used.
    pub metadata: Metadata,
    /// A list of components included in the BOM.
    pub components: Vec<Component>,
}

/// Metadata about the Bill of Materials.
#[derive(Debug, Serialize)]
pub struct Metadata {
    /// The timestamp when the BOM was created.
    pub timestamp: String,
    /// The tools used to generate the BOM.
    pub tools: Vec<Tool>,
    /// The main component that this BOM describes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<Component>,
}

/// Information about a tool used to generate the BOM.
#[derive(Debug, Serialize)]
pub struct Tool {
    /// The vendor of the tool.
    pub vendor: String,
    /// The name of the tool.
    pub name: String,
    /// The version of the tool.
    pub version: String,
}

/// Represents a component (e.g., a package or application) in the BOM.
#[derive(Debug, Serialize)]
pub struct Component {
    /// The type of the component (e.g., "application", "library").
    #[serde(rename = "type")]
    pub component_type: String,
    /// The group or namespace of the component.
    pub group: String,
    /// The name of the component.
    pub name: String,
    /// The version of the component.
    pub version: String,
    /// A brief description of the component.
    pub description: String,
    /// The licenses associated with the component.
    pub licenses: Vec<LicenseChoice>,
    /// The Package URL (purl) for the component.
    pub purl: String,
}

/// A choice of license for a component.
#[derive(Debug, Serialize)]
pub struct LicenseChoice {
    /// The license details.
    pub license: License,
}

/// Details about a license.
#[derive(Debug, Serialize)]
pub struct License {
    /// The name of the license.
    pub name: String,
}

impl Bom {
    /// Creates a new `Bom` with default metadata, including a unique serial number
    /// and the current timestamp.
    pub fn new() -> Self {
        Self::with_tool_version(env!("CARGO_PKG_VERSION").to_string())
    }

    /// Creates a new `Bom` with a specific tool version.
    pub fn with_tool_version(tool_version: String) -> Self {
        let now: DateTime<Utc> = Utc::now();
        Bom {
            bom_format: "CycloneDX".to_string(),
            spec_version: "1.7".to_string(),
            serial_number: format!("urn:uuid:{}", Uuid::new_v4()),
            version: 1,
            metadata: Metadata {
                timestamp: now.to_rfc3339(),
                tools: vec![Tool {
                    vendor: "cyclonedx-gentoo".to_string(),
                    name: "cyclonedx-gentoo".to_string(),
                    version: tool_version,
                }],
                component: None,
            },
            components: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bom_new() {
        let bom = Bom::new();
        assert_eq!(bom.bom_format, "CycloneDX");
        assert_eq!(bom.spec_version, "1.7");
        assert!(bom.serial_number.starts_with("urn:uuid:"));
        assert_eq!(bom.version, 1);
        assert_eq!(bom.metadata.tools.len(), 1);
        assert_eq!(bom.metadata.tools[0].name, "cyclonedx-gentoo");
        assert!(bom.components.is_empty());
    }

    #[test]
    fn test_serialization() {
        let mut bom = Bom::new();
        bom.components.push(Component {
            component_type: "library".to_string(),
            group: "dev-libs".to_string(),
            name: "openssl".to_string(),
            version: "3.0.12".to_string(),
            description: "Toolkit for SSL/TLS".to_string(),
            licenses: vec![LicenseChoice {
                license: License {
                    name: "Apache-2.0".to_string(),
                },
            }],
            purl: "pkg:gentoo/dev-libs/openssl@3.0.12".to_string(),
        });

        let json = serde_json::to_string(&bom).unwrap();
        assert!(json.contains("\"bomFormat\":\"CycloneDX\""));
        assert!(json.contains("\"specVersion\":\"1.7\""));
        assert!(json.contains("\"type\":\"library\""));
        assert!(json.contains("\"name\":\"openssl\""));
        assert!(json.contains("\"name\":\"Apache-2.0\""));
    }
}
