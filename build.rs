fn main() {
    let res = reqwest::blocking::get(
        "https://raw.githubusercontent.com/frk1/hazedumper/master/csgo.toml",
    ).expect("Downloading offsets");
    let content = res.text().expect("Convert response to text");
    
    _ = std::fs::create_dir("./.build");
    std::fs::write("./.build/offsets.toml", content)
        .expect("Write offsets to file");
}