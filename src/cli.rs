use crate::parser::Localization;

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};

/// Run the R REPL
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen,
    derive(Serialize, Deserialize)
)]
#[cfg_attr(not(feature = "wasm"), derive(clap::Parser))]
#[derive(Debug, Clone, Default)]
pub struct Cli {
    /// Localization to use for runtime
    #[cfg_attr(not(feature = "wasm"), arg(short, long, default_value = "en"))]
    pub locale: Localization,

    /// Show the extended warranty information at startup
    #[cfg_attr(not(feature = "wasm"), arg(long))]
    pub warranty: bool,
}
