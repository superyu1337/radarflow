use std::path::PathBuf;

use chrono::{Utc, TimeZone, Datelike};
use clap::Parser;
use memflow::prelude::Inventory;

use crate::structs::{Connector, Loglevel};

const PORT_RANGE: std::ops::RangeInclusive<usize> = 8000..=65535;
const POLL_RANGE: std::ops::RangeInclusive<usize> = 1..=1000;

#[derive(Parser)]
#[command(author, version = version(), about, long_about = None)]
pub struct Cli {
    /// Connector to use for DMA
    #[clap(value_enum, short, long, ignore_case = true, default_value_t = Connector::Qemu)]
    pub connector: Connector,

    /// Pcileech device name
    #[clap(long, default_value_t = String::from("FPGA"))]
    pub pcileech_device: String,

    /// Port to run Webserver on
    #[arg(short, long, default_value_t = 8000, value_parser = port_in_range)]
    pub port: u16,

    /// Path to serve on webserver
    #[arg(short, long, default_value = "./web", value_parser = valid_path)]
    pub web_path: PathBuf,

    /// How often per second the DMA thread should poll for data
    #[arg(short = 'r', long, default_value_t = 60, value_parser = poll_in_range)]
    pub poll_rate: u16,

    /// Loglevel verbosity
    #[arg(value_enum, long, short,  ignore_case = true, default_value_t = Loglevel::Warn)]
    pub loglevel: Loglevel,
}

fn version() -> String {
    let pkg_ver = env!("CARGO_PKG_VERSION");
    let git_hash = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
    let commit_date = option_env!("VERGEN_GIT_COMMIT_DATE").unwrap_or("unknown");
    let offsets = crate::dma::core::offsets::Offsets::new();
    let timestamp = Utc.timestamp_opt(offsets.timestamp as i64, 0).unwrap();
    let offsets_date = format!("{}-{:02}-{:02}", timestamp.year(), timestamp.month(), timestamp.day());
    let avail_cons = {
        let inventory = Inventory::scan();
        inventory.available_connectors().join(", ")
    };

    format!(" {pkg_ver} (rev {git_hash})\nCommit Date: {commit_date}\nOffset Date: {offsets_date}\nAvailable Connectors: {avail_cons}")
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