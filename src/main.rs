mod args;
mod dist;
mod error;
mod metadata;
mod pkgbuild;

use crate::error::Error;
use args::{get_args, CargoAurArgs};
use colored::*;
use dist::build_package;
use metadata::Config;
use pkgbuild::{pkgbuild, sha256sum};
use std::fs::{DirEntry, File};
use std::io::BufWriter;
use std::ops::Not;
use std::path::PathBuf;
use std::process::ExitCode;

type CargoAurResult = Result<(), Error>;

/// Licenses available from the Arch Linux `licenses` package.
///
/// That package contains other licenses, but I've excluded here those unlikely
/// to be used by Rust crates.
const LICENSES: &[&str] = &[
    "AGPL-3.0-only",
    "AGPL-3.0-or-later",
    "Apache-2.0",
    "BSL-1.0", // Boost Software License.
    "GPL-2.0-only",
    "GPL-2.0-or-later",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "LGPL-2.0-only",
    "LGPL-2.0-or-later",
    "LGPL-3.0-only",
    "LGPL-3.0-or-later",
    "MPL-2.0",   // Mozilla Public License.
    "Unlicense", // Not to be confused with "Unlicensed".
];

fn main() -> ExitCode {
    let args = get_args();

    if args.version {
        let version = env!("CARGO_PKG_VERSION");
        println!("{}", version);
        ExitCode::SUCCESS
    } else if let Err(e) = work(&args) {
        eprintln!("{} {}: {}", "::".bold(), "Error".bold().red(), e);
        ExitCode::FAILURE
    } else {
        println!("{} {}", "::".bold(), "Done.".bold().green());
        ExitCode::SUCCESS
    }
}

fn work(args: &CargoAurArgs) -> Result<(), Error> {
    // Ensure the target can actually be written to. Otherwise the `tar`
    // operation later on will fail.
    std::fs::create_dir_all("target/cargo-aur")?;

    let config = Config::new()?;

    // Warn if the user if still using the old metadata definition style.
    if let Some(metadata) = config.package.metadata.as_ref() {
        if metadata.depends.is_empty().not() || metadata.optdepends.is_empty().not() {
            p("Use of [package.metadata] is deprecated. Please specify extra dependencies under [package.metadata.aur].".bold().yellow());
        }
    }

    if args.dryrun.not() {
        let licenses = license_files()?;
        if !licenses.is_empty() {
            p("LICENSE file will be installed manually.".bold().yellow());
        };

        let gen_file = build_package(
            args.musl,
            &PathBuf::from("target/cargo-aur"),
            &config,
            &licenses,
        )?;

        let sha256: String = sha256sum(gen_file)?;

        // Write the PKGBUILD.
        let file = BufWriter::new(File::create("target/cargo-aur/PKGBUILD")?);
        pkgbuild(file, &config, &sha256, &licenses)?;
    }

    Ok(())
}

/// The path to the `LICENSE` file.
fn license_files() -> Result<Vec<DirEntry>, Error> {
    let licenses = std::fs::read_dir(".")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let binding = entry.file_name();
            let name = binding.to_str().unwrap_or_default();
            name.starts_with("LICENSE") && !LICENSES.contains(&name)
        })
        .collect::<Vec<_>>();
    if licenses.is_empty() {
        return Err(Error::MissingLicense);
    }
    Ok(licenses)
}

pub fn p(msg: ColoredString) {
    println!("{} {}", "::".bold(), msg)
}

