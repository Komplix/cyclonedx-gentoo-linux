mod cyclonedx;

use crate::cyclonedx::{create_bom, create_component};
use clap::{Arg, Command};
use std::path::Path;
use vardbpkg::parse_vardb;
use cyclonedx_bom::models::bom::Bom;
use cyclonedx_bom::models::component::Components;

/// Default path to the database on Gentoo Linux.
const DEFAULT_VAR_DB_PKG_PATH: &str = "/var/db/pkg";

/// Command-line arguments for the tool.
#[derive(Debug)]
struct Args {
    /// Optional group value for the top-level component.
    group: Option<String>,
    /// Optional path to an alternative var pkg directory.
    dir: Option<String>,
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
        dir: matches.get_one::<String>("dir").cloned(),
        name: matches.get_one::<String>("name").cloned(),
        only_master: matches.get_flag("only-master"),
        version: matches.get_one::<String>("version").cloned(),
    };

    let bom = generate_bom(&args, None)?;

    let mut output = Vec::new();
    bom.output_as_json_v1_5(&mut output).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    println!("{}", String::from_utf8_lossy(&output));

    Ok(())
}

fn generate_bom(args: &Args, tool_version: Option<String>) -> std::io::Result<Bom> {
    let db_path = args.dir.as_deref().unwrap_or(DEFAULT_VAR_DB_PKG_PATH);
    let packages = parse_vardb(Path::new(db_path));

    let mut bom = create_bom(tool_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()));

    if args.group.is_some() || args.name.is_some() || args.version.is_some() {
        let metadata_component = create_component(
            "application",
            args.group.as_deref().unwrap_or_default(),
            args.name.as_deref().unwrap_or_default(),
            args.version.as_deref().unwrap_or_default(),
            "",
            Vec::new(),
            "",
            Vec::new(),
        );
        if let Some(ref mut metadata) = bom.metadata {
            metadata.component = Some(metadata_component);
        }
    }

    if !args.only_master {
        let mut components = Vec::new();
        for pkg in packages {
            let mut licenses = Vec::new();
            for lic in pkg.license.split(' ') {
                if !lic.is_empty() {
                    licenses.push(lic.to_string());
                }
            }

            let mut homepages = Vec::new();
            for hp in pkg.homepage.split(' ') {
                if !hp.is_empty() {
                    homepages.push(hp.to_string());
                }
            }

            let component = create_component(
                "library",
                &pkg.category,
                &pkg.package,
                &pkg.version,
                &pkg.description,
                licenses,
                &format!(
                    "pkg:gentoo/{}/{}@{}",
                    pkg.category, pkg.package, pkg.version
                ),
                homepages,
            );
            components.push(component);
        }
        bom.components = Some(Components(components));
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
            Arg::new("dir")
                .short('d')
                .long("dir")
                .value_name("DIR")
                .help("(Optional) Use this directory as input instead of standard /var/db/pkg.")
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

        let matches =
            cli().get_matches_from(vec!["cyclonedx-gentoo", "-g", "mygroup", "--only-master"]);
        assert_eq!(matches.get_one::<String>("group").unwrap(), "mygroup");
        assert!(matches.get_flag("only-master"));
        assert!(matches.get_one::<String>("name").is_none());
    }

    #[test]
    fn test_generate_bom_from_mock_filesystem() {
        use std::fs;
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path();

        // Create a mock package: app-misc/mock-pkg-1.2.3
        let pkg_dir = db_path.join("app-misc").join("mock-pkg-1.2.3");
        fs::create_dir_all(&pkg_dir).unwrap();

        fs::write(pkg_dir.join("CATEGORY"), "app-misc\n").unwrap();
        fs::write(pkg_dir.join("PF"), "mock-pkg-1.2.3\n").unwrap();
        fs::write(pkg_dir.join("DESCRIPTION"), "A mock package for testing\n").unwrap();
        fs::write(pkg_dir.join("LICENSE"), "MIT Apache-2.0\n").unwrap();
        fs::write(pkg_dir.join("HOMEPAGE"), "https://example.com/mock-pkg\n").unwrap();

        let args = Args {
            group: None,
            dir: Some(db_path.to_str().unwrap().to_string()),
            name: None,
            only_master: false,
            version: None,
        };

        let bom = generate_bom(&args, Some("0.1.0".to_string())).unwrap();
        // Verify components
        let components = bom.components.as_ref().expect("Should have components");
        assert_eq!(components.0.len(), 1);

        let pkg = &components.0[0];
        assert_eq!(pkg.name.to_string(), "mock-pkg");
        assert_eq!(pkg.version.as_ref().unwrap().to_string(), "1.2.3");
        assert_eq!(pkg.group.as_ref().unwrap().to_string(), "app-misc");
        assert_eq!(pkg.description.as_ref().unwrap().to_string(), "A mock package for testing");
        
        // Verify licenses
        let licenses = pkg.licenses.as_ref().expect("Should have licenses");
        assert_eq!(licenses.0.len(), 2);
        // Note: cyclonedx-bom might store them as expressions if we use LicenseChoice::expression
        // Looking at cyclonedx.rs: license_list.push(LicenseChoice::expression(&lic_name));
        
        // Verify PURL
        assert_eq!(pkg.purl.as_ref().unwrap().to_string(), "pkg:gentoo/app-misc%2Fmock-pkg@1.2.3");

        // Verify External References (Homepage)
        let external_references = pkg.external_references.as_ref().expect("Should have external references");
        assert_eq!(external_references.0.len(), 1);
        assert_eq!(external_references.0[0].url.to_string(), "https://example.com/mock-pkg");
    }

}
