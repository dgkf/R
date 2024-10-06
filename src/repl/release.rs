use crate::cli::Experiment;
use crate::parser::Localization;
use crate::session::Session;
use std::sync::LazyLock;
use strum::IntoEnumIterator;

pub const RELEASE_NAME: &str = "Wonder Where We Land";
pub const GIT_HASH: &str = env!("GIT_HASH");
pub const RELEASE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const YEAR: &str = "2024";

pub const COPYRIGHT_LONG_FLAG: &str = "--warranty";
static COPYRIGHT: LazyLock<String> = LazyLock::new(|| {
    format!(
        "Copyright (C) {YEAR} R Authors
  
This program comes with ABSOLUTELY NO WARRANTY. This is free software, 
and you are welcome to redistribute it under certain conditions. For more
information, restart with `{COPYRIGHT_LONG_FLAG}`."
    )
});

static COPYRIGHT_LONG: LazyLock<String> = LazyLock::new(|| {
    format!(
        "Copyright (C) {YEAR} R Authors
  
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.  
  
This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.  
  
You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>."
    )
});

pub const AVAILABLE_FUNCTIONS: &str =
    "\nSee a list of implemented functions using `names(parent())`\n";

#[allow(clippy::const_is_empty)]
pub fn session_header(session: &Session) -> String {
    let dev = if !GIT_HASH.is_empty() {
        format!(" (dev {:.8})", GIT_HASH)
    } else {
        String::from("")
    };

    let license_info: &str = if session.warranty {
        COPYRIGHT_LONG.as_str()
    } else {
        COPYRIGHT.as_str()
    };

    let experiments: String = {
        let exp_strs: Vec<String> = Experiment::iter()
            .map(|exp| {
                if session.experiments.contains(&exp) {
                    format!("  [x] {exp:?}")
                } else {
                    format!("  [ ] {exp:?}")
                }
            })
            .collect();

        format!("\nExperiments:\n{}", exp_strs.join("\n"))
    };

    let langname = if session.locale == Localization::Pirate {
        "Arr"
    } else {
        "R"
    };

    format!("{langname} version {RELEASE_VERSION} -- \"{RELEASE_NAME}\"{dev}\n{license_info}\n{experiments}\n{AVAILABLE_FUNCTIONS\n}",)
}
