use std::process::Command;

fn main() {
    inject_git_commit().expect("Could not read curent git commit");
}

fn inject_git_commit() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    let git_hash = String::from_utf8(output.stdout)?;
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    Ok(())
}
