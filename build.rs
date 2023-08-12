use std::error::Error;
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let res = reqwest::blocking::get(
        "https://raw.githubusercontent.com/frk1/hazedumper/master/csgo.toml",
    ).expect("Downloading offsets");
    let content = res.text().expect("Convert response to text");
    
    _ = std::fs::create_dir("./.build");
    std::fs::write("./.build/offsets.toml", content)
        .expect("Write offsets to file");

    EmitBuilder::builder()
        .git_sha(true)
        .git_commit_date()
        .cargo_debug()
        .cargo_target_triple()
        .rustc_semver()
        .rustc_llvm_version()
        .emit()?;
    Ok(())
}