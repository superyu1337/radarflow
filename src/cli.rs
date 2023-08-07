use std::path::PathBuf;

use clap::Parser;

const PORT_RANGE: std::ops::RangeInclusive<usize> = 8000..=65535;
const POLL_RANGE: std::ops::RangeInclusive<usize> = 1..=1000;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value_t = 8000, value_parser = port_in_range)]
    pub port: u16,

    #[arg(short, long, default_value = "./web", value_parser = valid_path)]
    pub web_path: PathBuf,

    #[arg(short = 'r', long, default_value_t = 60)]
    #[arg(value_parser = poll_in_range)]
    pub poll_rate: u16,

    #[arg(long, value_enum, default_value = log::LevelFilter::Warn.as_str())]
    pub loglevel: log::LevelFilter,
}

fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a port number"))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}

fn valid_path(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);

    if !path.exists() {
        return Err("Path does not exist".to_string())
    }

    if !path.is_dir() {
        return Err("Path is not a directory".to_string())
    }

    Ok(path)
}

fn poll_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a valid number"))?;
    if POLL_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "not in range {}-{}",
            POLL_RANGE.start(),
            POLL_RANGE.end()
        ))
    }
}