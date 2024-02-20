use crate::parser::Localization;

#[cfg_attr(not(feature = "wasm"), derive(clap::Parser))]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Cli {
    #[cfg_attr(not(feature = "wasm"), arg(short, long, default_value_t = Localization::En))]
    pub locale: Localization,

    #[cfg_attr(not(feature = "wasm"), arg(long))]
    pub warranty: bool,
}
