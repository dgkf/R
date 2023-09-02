use std::{fs, env};
use regex::{RegexBuilder, Captures};

/// Log to cargo's warning output
///
/// Only enabled if environment variable "LOG" is set.
///
/// ```
/// LOG=1 cargo build
/// ```
///
macro_rules! log {
    ($($tokens: tt)*) => {
        env::var("LOG")
            .and_then(|_| Ok(print!("cargo:warning={}\n", format!($($tokens)*))))
            .unwrap_or_default();
    }
}

fn scrape_builtins(paths: String) -> Result<Vec<(String, String)>, ()> {

    let mut builtins = vec![];
    let paths = std::fs::read_dir(paths).map_err(|_| ())?;

    let re = RegexBuilder::new(r#"#\[builtin\(.*\bsym\s*=\s*\"(.*)\".*\)\].*struct\s+(\w+?)"#)
        .multi_line(true)
        .dot_matches_new_line(true)
        .swap_greed(true)
        .crlf(true)
        .build()
        .expect("Regex is malformed");

    for path in paths {
        let Ok(file) = path else {
            continue            
        };

        match file.file_type() {
            Ok(filetype) if filetype.is_file() => {
                let content = fs::read_to_string(file.path()).expect("File not found.");
                log!("Scanning file {:?}", file.path().into_os_string());
                for (_, [sym, ty]) in re.captures_iter(&content).map(|c| c.extract()) {
                    log!("  - {ty} as '{sym}'");
                    builtins.push((String::from(sym), String::from(ty)))
                };
            },
            Ok(filetype) if filetype.is_dir() => {
                let dirpath = file.path().into_os_string().into_string().map_err(|_| ())?;
                let mut dirbuiltins = scrape_builtins(dirpath)?;
                builtins.append(&mut dirbuiltins);
            },
            _ => continue
        }
        
    }

    Ok(builtins)
}

fn update_builtins_table(path: String, builtins: Vec<(String, String)>) -> Result<(), ()> {
    let content = fs::read_to_string(&path)
        .expect("Unable to read builtins table file.");

    let re = RegexBuilder::new(r#"(// builtins start)\s*(\n\s*?).*(\n\s*?// builtins end)"#)
        .multi_line(true)
        .dot_matches_new_line(true)
        .swap_greed(true)
        .crlf(true)
        .build()
        .expect("Regex is malformed");
 
    let content = re.replace(&content, |cap: &Captures| {
        let mut res = String::from("");
        let (_, [head, ws, tail]) = cap.extract();
        res.push_str(head);
        for (sym, ty) in builtins.clone() {
            res.push_str(format!(r#"{ws}("{sym}", Box::new({ty}) as Box<dyn Builtin>),"#).as_str());
        }
        res.push_str(tail);
        res
    });

    log!("Updating {path} ... ");

    fs::write(&path, content.to_string()).expect("Error encountered while updating builtins table");
    Ok(())
}

fn git_hash() -> String {
    use std::process::Command;
    let unknown = String::from("unknown");

    let changes = !Command::new("git")
        .args(&["diff", "--cached", "--exit-code"])
        .status()
        .expect("Error encountered while checking for git changes")
        .success();

    if changes { return unknown }

    Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg("HEAD")
        .output()
        .map_or(Ok(unknown), |o| String::from_utf8(o.stdout))
        .expect("Error encountered while reading it output")
}

fn main() -> Result<(), ()> {
    // embed git hash as environment variable GIT_SHA for use in header
    println!("cargo:rustc-env=GIT_HASH={}", git_hash());
    
    // iterate through callable module files and scan for uses of 
    // `#[builtin(.. sym = "sym" ..)]`
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/callable");
    let builtins = scrape_builtins("src/callable".into())?;    
    update_builtins_table("src/callable/builtins.rs".into(), builtins)?;

    Ok(())
}