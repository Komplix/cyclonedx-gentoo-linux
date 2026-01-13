mod cyclonedx;

use crate::cyclonedx::{Bom, Component, License, LicenseChoice};
use clap::{Arg, Command};
use std::path::Path;
use vardbpkg::parse_vardb;

/// Default path to the eix database on Gentoo Linux.
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

    println!("{}", serde_json::to_string_pretty(&bom)?);

    Ok(())
}

fn generate_bom(args: &Args, tool_version: Option<String>) -> std::io::Result<Bom> {
    let db_path = args.dir.as_deref().unwrap_or(DEFAULT_VAR_DB_PKG_PATH);
    let packages = parse_vardb(Path::new(db_path));

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
        for pkg in packages {
            let mut licenses = Vec::new();
            for lic in pkg.license.split(' ') {
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
                group: pkg.category.clone(),
                name: pkg.package.clone(),
                version: pkg.version.clone(),
                description: pkg.description.clone(),
                licenses,
                purl: format!(
                    "pkg:gentoo/{}/{}@{}",
                    pkg.category, pkg.package, pkg.version
                ),
            };
            bom.components.push(component);
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
}
