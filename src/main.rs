mod cyclonedx;

use crate::cyclonedx::{Bom, Component, License, LicenseChoice};
use clap::{Arg, Command};
use eix::{Database, PackageReader};

/// Default path to the eix database on Gentoo Linux.
const DEFAULT_EIX_DB_PATH: &str = "/var/cache/eix/portage.eix";

/// Command-line arguments for the tool.
#[derive(Debug)]
struct Args {
    /// Optional group value for the top-level component.
    group: Option<String>,
    /// Optional path to an alternative eix database file.
    file: Option<String>,
    /// Optional name for the top-level component.
    name: Option<String>,
    /// If true, only the top-level component is included in the output.
    only_master: bool,
    /// Optional version for the top-level component.
    version: Option<String>,
}

fn main() -> std::io::Result<()> {
    let matches = cli().get_matches();

    let args = Args {
        group: matches.get_one::<String>("group").cloned(),
        file: matches.get_one::<String>("file").cloned(),
        name: matches.get_one::<String>("name").cloned(),
        only_master: matches.get_flag("only-master"),
        version: matches.get_one::<String>("version").cloned(),
    };

    let bom = generate_bom(&args, None)?;

    println!("{}", serde_json::to_string_pretty(&bom)?);

    Ok(())
}

fn generate_bom(args: &Args, tool_version: Option<String>) -> std::io::Result<Bom> {
    let db_path = args.file.as_deref().unwrap_or(DEFAULT_EIX_DB_PATH);
    let mut db = Database::open_read(db_path)?;
    let header = db.read_header(0)?;
    let mut reader = PackageReader::new(db, header);

    let mut bom = if let Some(version) = tool_version {
        Bom::with_tool_version(version)
    } else {
        Bom::new()
    };

    if args.group.is_some() || args.name.is_some() || args.version.is_some() {
        bom.metadata.component = Some(Component {
            component_type: "application".to_string(),
            group: args.group.clone().unwrap_or_default(),
            name: args.name.clone().unwrap_or_default(),
            version: args.version.clone().unwrap_or_default(),
            description: "".to_string(),
            licenses: Vec::new(),
            purl: "".to_string(),
        });
    }

    if !args.only_master {
        while reader.next_category()? {
            let category = reader.current_category().to_string();
            while let Some(pkg) = reader.read_package()? {
                for v in &pkg.versions {
                    if v.is_installed() {
                        let mut licenses = Vec::new();
                        for lic in pkg.licenses.split(' ') {
                            if !lic.is_empty() {
                                licenses.push(LicenseChoice {
                                    license: License {
                                        name: lic.to_string(),
                                    },
                                });
                            }
                        }

                        let component = Component {
                            component_type: "library".to_string(),
                            group: category.clone(),
                            name: pkg.name.clone(),
                            version: v.version_string.clone(),
                            description: pkg.description.clone(),
                            licenses,
                            purl: format!(
                                "pkg:gentoo/{}/{}@{}?repository={}",
                                category, pkg.name, v.version_string, v.reponame
                            ),
                        };
                        bom.components.push(component);
                    }
                }
            }
        }
    }

    Ok(bom)
}

fn cli() -> Command {
    Command::new("cyclonedx-gentoo")
        .about("Generates SBOM in CycloneDX format for Gentoo-Linux Portage Packet database")
        .override_usage("cyclonedx-gentoo [OPTIONS]")
        .disable_help_flag(true)
        .arg(
            Arg::new("group")
                .short('g')
                .long("group")
                .value_name("GROUP")
                .help("(Optional) Group value to assign to top level component.")
                .num_args(1),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .help("will print out the command line options.")
                .action(clap::ArgAction::Help),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("(Optional) Use this eix-file as input instead of standard file.")
                .num_args(1),
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("(Optional) Name value to assign to top level component.")
                .num_args(1),
        )
        .arg(
            Arg::new("only-master")
                .short('m')
                .long("only-master")
                .help("(Optional) Will only capture master component. Will not include any components in the list of Components.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .value_name("VERSION")
                .help("(Optional) Version value to assign to top level component.")
                .num_args(1),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_config() {
        let cmd = cli();
        assert_eq!(cmd.get_name(), "cyclonedx-gentoo");
        
        let matches = cli().get_matches_from(vec!["cyclonedx-gentoo", "-g", "mygroup", "--only-master"]);
        assert_eq!(matches.get_one::<String>("group").unwrap(), "mygroup");
        assert!(matches.get_flag("only-master"));
        assert!(matches.get_one::<String>("name").is_none());
    }

    #[test]
    fn test_generate_bom_from_testdata() {
        let args = Args {
            group: Some("test-group".to_string()),
            file: Some("testdata/portage.eix".to_string()),
            name: Some("test-name".to_string()),
            only_master: false,
            version: Some("1.2.3".to_string()),
        };

        let result = generate_bom(&args, None);
        assert!(result.is_ok(), "BOM generation failed: {:?}", result.err());
        
        let bom = result.unwrap();
        assert_eq!(bom.metadata.component.as_ref().unwrap().name, "test-name");
        assert_eq!(bom.metadata.component.as_ref().unwrap().group, "test-group");
        assert_eq!(bom.metadata.component.as_ref().unwrap().version, "1.2.3");

        // The portage.eix in testdata should contain some installed packages
        assert!(!bom.components.is_empty(), "BOM should contain components from testdata/portage.eix");
        
        // Check a known package or just the structure
        let first_comp = &bom.components[0];
        assert!(!first_comp.name.is_empty());
        assert!(!first_comp.version.is_empty());
        assert!(first_comp.purl.starts_with("pkg:gentoo/"));
    }

    #[test]
    fn test_generate_bom_only_master_from_testdata() {
        let args = Args {
            group: Some("test-group".to_string()),
            file: Some("testdata/portage.eix".to_string()),
            name: Some("test-name".to_string()),
            only_master: true,
            version: Some("1.2.3".to_string()),
        };

        let result = generate_bom(&args, None);
        assert!(result.is_ok());
        
        let bom = result.unwrap();
        assert!(bom.components.is_empty(), "BOM components should be empty when only-master is true");
    }

    #[test]
    fn test_bom_against_json() {
        let args = Args {
            group: None,
            file: Some("testdata/portage.eix".to_string()),
            name: None,
            only_master: false,
            version: Some("4.7.11".to_string())
        };

        let bom = generate_bom(&args, Some("0.8.15".to_string())).expect("Failed to generate BOM");

        let bom_json = serde_json::to_value(&bom).expect("Failed to serialize BOM");

        let expected_json_str = std::fs::read_to_string("testdata/portage.json").expect("Failed to read portage.json");
        let expected_json: serde_json::Value = serde_json::from_str(&expected_json_str).expect("Failed to parse portage.json");

        // Compare important fields
        assert_eq!(bom_json["bomFormat"], expected_json["bomFormat"]);
        assert_eq!(bom_json["specVersion"], expected_json["specVersion"]);
        assert_eq!(bom_json["version"], expected_json["version"], "Global version differs");
        assert_eq!(bom.metadata.component.as_ref().unwrap().version, "4.7.11");


        // Compare tools version
        assert_eq!(bom_json["metadata"]["tools"][0]["version"], expected_json["metadata"]["tools"][0]["version"], "Tool version differs");

        // Compare components
        let components = bom_json["components"].as_array().expect("Components should be an array");
        let expected_components = expected_json["components"].as_array().expect("Expected components should be an array");
        
        assert_eq!(components.len(), expected_components.len(), "Number of components differs");

        for (i, comp) in components.iter().enumerate() {
            let exp_comp = &expected_components[i];
            assert_eq!(comp["name"], exp_comp["name"], "Component {} name differs", i);
            assert_eq!(comp["version"], exp_comp["version"], "Component {} version differs", i);
            assert_eq!(comp["group"], exp_comp["group"], "Component {} group differs", i);
            assert_eq!(comp["purl"], exp_comp["purl"], "Component {} purl differs", i);
            assert_eq!(comp["type"], exp_comp["type"], "Component {} type differs", i);
            
        }
    }
}
