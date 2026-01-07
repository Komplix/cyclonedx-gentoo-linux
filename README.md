# cyclonedx-gentoo
[<img alt="github" src="https://img.shields.io/badge/github-Komplix%2Fcyclonedx--gentoo--linux-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/Komplix/cyclonedx-gentoo-linux)
[![Build](https://github.com/Komplix/cyclonedx-gentoo-linux/actions/workflows/build.yml/badge.svg)](https://github.com/Komplix/cyclonedx-gentoo-linux/actions/workflows/build.yml)
![maintenance-status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

Generates a Software Bill of Materials (SBOM) in CycloneDX format (JSON) for Gentoo Linux by parsing the Portage package database via `eix`.

## Description

`cyclonedx-gentoo` is a command-line tool that scans your installed Gentoo packages and produces a CycloneDX SBOM. It uses the `eix` database for fast access to package information.

The output is a [CycloneDX](https://cyclonedx.org/) v1.5 JSON document sent to `stdout`.

## Prerequisites

- Gentoo Linux
- `app-portage/eix` installed and database updated (`eix-update`)
- Rust toolchain (to build from source)

## Installation

```bash
cargo install --path .
```

## Usage

By default, the tool looks for the eix database at `/var/cache/eix/portage.eix`.

```bash
cyclonedx-gentoo [OPTIONS]
```

### Options

- `-g, --group <arg>`: (Optional) Group value to assign to the top-level component.
- `-n, --name <arg>`: (Optional) Name value to assign to the top-level component.
- `-v, --version <arg>`: (Optional) Version value to assign to the top-level component.
- `-f, --file <arg>`: (Optional) Use a specific eix-file as input instead of the standard one.
- `-m, --only-master`: (Optional) Only capture the master component (metadata). Will not include any installed packages in the components list.
- `-h, --help`: Print out the command line options.

### Example

Generate an SBOM for the current system:

```bash
cyclonedx-gentoo > sbom.json
```

Generate an SBOM with a specific top-level component name and version:

```bash
cyclonedx-gentoo --name "My-Gentoo-System" --version "2024.01" > sbom.json
```

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
