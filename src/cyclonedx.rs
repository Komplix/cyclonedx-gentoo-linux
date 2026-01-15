//! CycloneDX SBOM models and implementation using cyclonedx-bom crate.

use cyclonedx_bom::models::bom::Bom as CdxBom;
use cyclonedx_bom::models::component::{Component as CdxComponent, Classification};
use cyclonedx_bom::models::external_reference::{
    ExternalReference, ExternalReferenceType, ExternalReferences, Uri as ExternalReferenceUri,
};
use cyclonedx_bom::models::metadata::Metadata as CdxMetadata;
use cyclonedx_bom::models::tool::{Tool as CdxTool, Tools};
use cyclonedx_bom::models::license::{LicenseChoice, Licenses};
use cyclonedx_bom::prelude::*;
use uuid::Uuid;

pub fn create_bom(tool_version: String) -> CdxBom {
    let mut bom = CdxBom::default();
    bom.spec_version = SpecVersion::V1_5;
    bom.serial_number = Some(UrnUuid::from(Uuid::new_v4()));
    bom.version = 1;

    let mut metadata = CdxMetadata::default();
    metadata.timestamp = Some(DateTime::now().unwrap());
    
    let tool = CdxTool {
        vendor: Some(NormalizedString::new("cyclonedx-gentoo")),
        name: Some(NormalizedString::new("cyclonedx-gentoo")),
        version: Some(NormalizedString::new(&tool_version)),
        ..Default::default()
    };
    metadata.tools = Some(Tools::List(vec![tool]));
    
    bom.metadata = Some(metadata);
    bom
}

pub fn create_component(
    component_type: &str,
    group: &str,
    name: &str,
    version: &str,
    description: &str,
    licenses: Vec<String>,
    purl_str: &str,
    homepages: Vec<String>,
) -> CdxComponent {
    let mut component = CdxComponent::new(
        match component_type {
            "application" => Classification::Application,
            "library" => Classification::Library,
            _ => Classification::Library,
        },
        name,
        version,
        None,
    );

    if !group.is_empty() {
        component.group = Some(NormalizedString::new(group));
    }
    
    if !description.is_empty() {
        component.description = Some(NormalizedString::new(description));
    }

    if !purl_str.is_empty() {
        // pkg:gentoo/dev-libs/openssl@3.0.12
        if let Some(rest) = purl_str.strip_prefix("pkg:") {
            if let Some((p_type, rest)) = rest.split_once('/') {
                if let Some((p_name, p_version)) = rest.split_once('@') {
                     component.purl = Some(Purl::new(p_type, p_name, p_version).unwrap());
                }
            }
        }
    }

    if !licenses.is_empty() {
        let mut license_list = Vec::new();
        for lic_name in licenses {
            license_list.push(LicenseChoice::expression(&lic_name));
        }
        component.licenses = Some(Licenses(license_list));
    }
    
    if !homepages.is_empty() {
        let mut external_references = Vec::new();
        for homepage in homepages {
            external_references.push(ExternalReference {
                external_reference_type: ExternalReferenceType::Website,
                url: ExternalReferenceUri::Url(Uri::new(&homepage)),
                comment: None,
                hashes: None,
            });
        }
        if !external_references.is_empty() {
            component.external_references = Some(ExternalReferences(external_references));
        }
    }

    component
}
