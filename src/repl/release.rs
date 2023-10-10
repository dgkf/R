pub const RELEASE_NAME: &str = "Art Smock";

pub const GIT_HASH: &str = env!("GIT_HASH");
pub const RELEASE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const YEAR: &str = "2023";

pub const COPYRIGHT_LONG_INST: &str = "cargo run -- --warranty";
lazy_static::lazy_static! {
    static ref COPYRIGHT: String = format!(
"Copyright (C) {YEAR} {AUTHORS}  
  
This program comes with ABSOLUTELY NO WARRANTY. This is free software, 
and you are welcome to redistribute it under certain conditions. For more
information, restart with:

    {COPYRIGHT_LONG_INST}");
}

lazy_static::lazy_static! {
    static ref COPYRIGHT_LONG: String = format!(
"Copyright (C) {YEAR} {AUTHORS}  
  
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.  
  
This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.  
  
You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.");
}

pub fn session_header() -> String {
    let args: Vec<String> = std::env::args().collect();

    let dev = if !GIT_HASH.is_empty() {
        format!(" (dev {:.8})", GIT_HASH)
    } else {
        String::from("")
    };

    let license_info: &str = if args.contains(&"--warranty".to_string()) {
        COPYRIGHT_LONG.as_str()
    } else {
        COPYRIGHT.as_str()
    };

    format!("R version {RELEASE_VERSION} -- \"{RELEASE_NAME}\"{dev}\n{license_info}\n",)
}
