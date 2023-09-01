// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/callable**/*");

    // iterate through callable files and scan for uses of `#[register_primitive]`
}