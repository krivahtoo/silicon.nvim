use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["describe", "--tags"])
        .output()
        .expect("git command not found.");
    if output.status.success() {
        println!(
            "cargo:rustc-env=SILICON_VERSION={}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}
