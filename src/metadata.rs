use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub(crate) package: Package,
    #[serde(default)]
    pub(crate) bin: Vec<Binary>,
}

impl Config {
    pub fn new() -> Result<Config, Error> {
        let content = std::fs::read_to_string("Cargo.toml")?;
        let proj: Config = toml::from_str(&content)?;
        Ok(proj)
    }
    /// The name of the compiled binary that should be copied to the tarball.
    pub fn binary_name(&self) -> &str {
        self.bin
            .first()
            .map(|bin| bin.name.as_str())
            .unwrap_or(self.package.name.as_str())
    }
}

#[derive(Deserialize, Debug)]
pub struct Package {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) authors: Vec<String>,
    pub(crate) description: String,
    pub(crate) homepage: String,
    pub(crate) repository: String,
    pub(crate) license: String,
    pub(crate) metadata: Option<Metadata>,
}

#[derive(Deserialize, Debug)]
pub struct AUR {
    #[serde(default)]
    pub(crate) depends: Vec<String>,
    #[serde(default)]
    pub(crate) optdepends: Vec<String>,
}

impl Package {
    /// The name of the tarball that should be produced from this `Package`.
    pub fn tarball(&self) -> String {
        format!(
            "{}-{}-x86_64.tar.gz",
            self.name, self.version
        )
    }

    pub fn git_host(&self) -> Option<GitHost> {
        if self.repository.starts_with("https://github") {
            Some(GitHost::Github)
        } else if self.repository.starts_with("https://gitlab") {
            Some(GitHost::Gitlab)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub enum GitHost {
    #[default]
    Github,
    Gitlab,
}

impl GitHost {
    pub fn source(&self, package: &Package) -> String {
        match self {
            GitHost::Github => format!(
                "{}/releases/download/$pkgver/{}-$pkgver-x86_64.tar.gz",
                package.repository, package.name
            ),
            GitHost::Gitlab => format!(
                "{}/-/archive/$pkgver/{}-$pkgver-x86_64.tar.gz",
                package.repository, package.name
            ),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Binary {
    pub(crate) name: String,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    /// Deprecated.
    #[serde(default)]
    pub(crate) depends: Vec<String>,
    /// Deprecated.
    #[serde(default)]
    pub(crate) optdepends: Vec<String>,
    /// > [package.metadata.aur]
    pub(crate) aur: Option<AUR>,
}

impl std::fmt::Display for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Reconcile which section to read extra dependency information from.
        // The format we hope the user is using is:
        //
        // > [package.metadata.aur]
        //
        // But version 1.5 originally supported:
        //
        // > [package.metadata]
        //
        // To avoid a sudden breakage for users, we support both definition
        // locations but favour the newer one.
        //
        // We print a warning to the user elsewhere if they're still using the
        // old way.
        let (deps, opts) = if let Some(aur) = self.aur.as_ref() {
            (aur.depends.as_slice(), aur.optdepends.as_slice())
        } else {
            (self.depends.as_slice(), self.optdepends.as_slice())
        };

        match deps {
            [middle @ .., last] => {
                write!(f, "depends=(")?;
                for item in middle {
                    write!(f, "\"{}\" ", item)?;
                }
                if !opts.is_empty() {
                    writeln!(f, "\"{}\")", last)?;
                } else {
                    write!(f, "\"{}\")", last)?;
                }
            }
            [] => {}
        }

        match opts {
            [middle @ .., last] => {
                write!(f, "optdepends=(")?;
                for item in middle {
                    write!(f, "\"{}\" ", item)?;
                }
                write!(f, "\"{}\")", last)?;
            }
            [] => {}
        }

        Ok(())
    }
}
