use gumdrop::{Options, ParsingStyle};

#[derive(Options)]
pub struct CargoAurArgs {
    /// Display this help message.
    pub help: bool,
    /// Display the current version of this software.
    pub version: bool,
    /// Unused.
    #[options(free)]
    pub args: Vec<String>,
    /// Use the MUSL build target to produce a static binary.
    pub musl: bool,
    /// Don't actually build anything.
    pub dryrun: bool,
}

pub fn get_args() -> CargoAurArgs {
    CargoAurArgs::parse_args_or_exit(ParsingStyle::AllOptions)
}
