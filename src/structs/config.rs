use std::{ops::RangeInclusive, fmt::Display};

use field_types::FieldName;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "ConfigShadow")]
pub struct Config {
    /// Port to run the http server on
    web_port: u16,

    /// Port to run the websocket on
    socket_port: u16,

    /// Path to static web files (index.html etc.)
    web_path: String,

    /// How often the DMA thread should poll for data in one second (in Hz)
    poll_rate: u16
}

impl Default for Config {
    fn default() -> Config {
        Config {
            web_port: 8000,
            socket_port: 8001,
            web_path: String::from("./web"),
            poll_rate: 60
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config> {
        let path = std::path::PathBuf::from(path);

        if path.exists() && path.is_file() {
            let str = std::fs::read_to_string(&path)?;
            let cfg = toml::from_str(&str)?;
            log::info!("Found valid config file at \"{}\"", path.to_string_lossy());
            Ok(cfg)
        } else {
            log::info!("Could not find config file at \"{}\", falling back to default config", path.to_string_lossy());
            let cfg = Config::default();

            let web_path = std::path::PathBuf::from(&cfg.web_path);

            if !web_path.exists() {
                return Err(
                    anyhow::Error::msg(format!("\"{}\" does not exist", web_path.to_string_lossy()))
                )
            } else if !web_path.is_dir() {
                return Err(
                    anyhow::Error::msg(format!("\"{}\" is not a directory", web_path.to_string_lossy()))
                )
            }

            Ok(cfg)
        }
    }

    pub fn web_port(&self) -> u16 {
            self.web_port
    }

    pub fn websocket_port(&self) -> u16 {
        self.socket_port
    }

    pub fn web_path(&self) -> &str {
        &self.web_path
    }

    pub fn poll_rate(&self) -> u16 {
        self.poll_rate
    }
}

#[derive(Deserialize, FieldName)]
#[serde(default)]
pub struct ConfigShadow {
    web_port: u16,
    socket_port: u16,
    web_path: String,
    poll_rate: u16
}

impl Default for ConfigShadow {
    fn default() -> ConfigShadow {
        ConfigShadow {
            web_port: 8000,
            socket_port: 8001,
            web_path: String::from("./web"),
            poll_rate: 60
        }
    }
}

fn check_in_range<T: Display + PartialOrd>(value_str: &str, value: T, range: RangeInclusive<T>) -> Result<()> {
    if range.contains(&value) {
        Ok(())
    } else {
        Err(
            anyhow::Error::msg(format!("Config option \"{}\" is out of range (min: {}, max: {}, val: {})", 
                value_str, range.start(), range.end(), value
            ))
        )
    }
}

fn check_valid_path(value_str: &str, path: &str) -> Result<()> {
    let path = std::path::PathBuf::from(path);
    
    if !path.exists() {
        return Err(
            anyhow::Error::msg(format!("Config option \"{}\" is not a valid path: \"{}\" does not exist", 
                value_str, path.to_string_lossy()
            ))
        )
    }

    if !path.is_dir() {
        return Err(
            anyhow::Error::msg(format!("Config option \"{}\" is not a valid path: \"{}\" is not a directory", 
                value_str, path.to_string_lossy()
            ))
        )
    }

    Ok(())
}

impl std::convert::TryFrom<ConfigShadow> for Config {
    type Error = anyhow::Error;

    fn try_from(shadow: ConfigShadow) -> Result<Self, Self::Error> {
        if shadow.web_port == shadow.socket_port {
            return Err(
                anyhow::Error::msg(format!("{} and {} cannot be the same",
                ConfigShadowFieldName::SocketPort.name(),
                ConfigShadowFieldName::WebPort.name()
            ))
            )
        }

        check_in_range(ConfigShadowFieldName::WebPort.name(), shadow.web_port, 8000..=65535)?;
        check_in_range(ConfigShadowFieldName::SocketPort.name(), shadow.web_port, 8000..=65535)?;
        check_in_range(ConfigShadowFieldName::PollRate.name(), shadow.poll_rate, 1..=1000)?;
        check_valid_path(ConfigShadowFieldName::WebPath.name(), &shadow.web_path)?;

        Ok(Config {
            web_port: shadow.web_port,
            web_path: shadow.web_path,
            socket_port: shadow.socket_port,
            poll_rate: shadow.poll_rate
        })
    }
}
