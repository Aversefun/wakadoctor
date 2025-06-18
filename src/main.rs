//! Wakatime config tester.
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use std::fmt::Display;

use clap::Parser;
use serde::Deserialize;

/// Wakatime configuration tester. Tests for presense of the wakatime CLI, validates API keys, and more.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of the wakatime config file.
    #[arg(short, long, default_value = "~/.wakatime.cfg")]
    config_location: String,
    /// Assume you AREN'T trying to use Hackatime.
    #[arg(short = 'w', long = "no-warn-default-waka", default_value_t = false)]
    no_warn_default_waka: bool,
    /// Assume you ARE trying to use a custom server.
    #[arg(short = 'u', long = "custom-server", default_value_t = false)]
    custom_server: bool,
    /// Do not attempt to send a heartbeat to test the server.
    #[arg(short = 'o', long = "offline", default_value_t = false)]
    offline: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
#[allow(clippy::missing_docs_in_private_items)]
struct WakaSettings {
    debug: bool,
    api_key: String,
    api_key_vault_cmd: String,
    api_url: String,
    hide_file_names: bool,
    hide_project_names: bool,
    hide_branch_names: bool,
    hide_dependencies: bool,
    hide_project_folder: bool,
}

#[derive(Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
struct WakaConfig {
    settings: WakaSettings,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(clippy::missing_docs_in_private_items)]
enum WakaHost {
    Hackatime,
    OldHackatime,
    Wakatime,
    Custom,
}

impl Display for WakaHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hackatime | Self::OldHackatime => f.write_str("Hackatime"),
            Self::Wakatime | Self::Custom => f.write_str("Wakatime"),
        }
    }
}

#[tokio::main]
async fn main() {
    let mut args = Args::parse();
    args.config_location = args
        .config_location
        .replace("~", &std::env::home_dir().unwrap().to_string_lossy());

    println!("Wakadoctor - Test your wakatime configuration");
    println!("Version {}", env!("CARGO_PKG_VERSION"));
    println!();

    let config: WakaConfig =
        match serde_ini::from_str(match &std::fs::read_to_string(args.config_location) {
            Ok(v) => {
                println!("✅ - Successfully read Wakatime config");
                v
            }
            Err(e) => {
                println!("❌ - Cannot read Wakatime config with error \"{e}\"");
                return;
            }
        }) {
            Ok(v) => {
                println!("✅ - Successfully parsed Wakatime config");
                v
            }
            Err(e) => {
                println!("❌ - Cannot parse Wakatime config with error \"{e}\"");
                return;
            }
        };

    let url = if config.settings.api_url.is_empty() {
        println!(
            "⚠️ - Wakatime API URL is not specified - assuming default (https://api.wakatime.com/api/v1)"
        );
        url::Url::parse("https://api.wakatime.com/api/v1").unwrap()
    } else {
        match url::Url::parse(&config.settings.api_url) {
            Ok(v) => {
                println!("✅ - Wakatime API URL is valid URL");
                v
            }
            Err(e) => {
                println!(
                    "❌ - Wakatime API URL is not valid URL (failed parsing with error {e})"
                );
                return;
            }
        }
    };

    let host = match url.host_str().unwrap_or_else(|| {
        println!("❌ - Wakatime API URL has null host");
        ""
    }) {
        "hackatime.hackclub.com" => {
            println!("✅ - Wakatime API host is Hackatime host");
            WakaHost::Hackatime
        }
        "waka.hackclub.com" => {
            println!("⚠️ - Wakatime API host is old Hackclub Wakatime host");
            WakaHost::OldHackatime
        }
        "api.wakatime.com" => {
            if args.no_warn_default_waka {
                println!("✅ - Wakatime API host is default Wakatime host");
            } else {
                println!(
                    "⚠️ - Wakatime API host is default Wakatime host (psst- disable this warning with --no-warn-default-waka)"
                );
            }
            WakaHost::Wakatime
        }
        "" => {
            return;
        }
        _ => {
            if args.custom_server {
                println!("✅ - Wakatime API host is custom server host");
            } else {
                println!(
                    "⚠️ - Wakatime API host is custom server host or invalid host (psst- disable this warning with --custom-server)"
                );
            }
            WakaHost::Custom
        }
    };

    match host {
        WakaHost::Hackatime => {
            if url.path() != "/api/hackatime/v1" {
                println!(
                    "❌ - Hackatime API path should be \"/api/hackatime/v1\", not \"{}\"",
                    url.path()
                );
                return;
            } else {
                println!("✅ - Hackatime API path is correct.");
            }
        }
        WakaHost::OldHackatime => {}
        WakaHost::Wakatime => {
            if url.path() != "/api/v1" {
                println!(
                    "❌ - Wakatime API path should be \"/api/v1\", not \"{}\"",
                    url.path()
                );
                return;
            } else {
                println!("✅ - Wakatime API path is correct.");
            }
        }
        WakaHost::Custom => {}
    }

    if url.scheme() != "https" {
        if url.scheme() == "http" {
            println!(
                "❌ - Wakatime API URL is unsecured HTTP"
            );
        } else {
            println!(
                "❌ - Wakatime API URL has unknown scheme \"{}\"",
                url.scheme()
            );
        }
        return;
    } else {
        println!(
            "✅ - Wakatime API URL is HTTPS"
        );
    }

    if config.settings.api_key.is_empty() {
        println!("❌ - No API key in file");
        return;
    } else if host == WakaHost::Hackatime {
        match uuid::Uuid::parse_str(&config.settings.api_key) {
            Ok(_) => {
                println!("✅ - Hackatime API key is in valid format");
            }
            Err(_) => {
                println!("❌ - Hackatime API key is NOT in valid format");
                return;
            }
        }
    } else if host == WakaHost::Wakatime {
        if config.settings.api_key.starts_with("waka_") {
            match uuid::Uuid::parse_str(&config.settings.api_key.replacen("waka_", "", 1)) {
                Ok(_) => {
                    println!("✅ - Wakatime API key is in valid format");
                }
                Err(_) => {
                    println!("❌ - Wakatime API key is NOT in valid format");
                    return;
                }
            }
        } else {
            println!("❌ - Wakatime API key is NOT in valid format");
            return;
        }
    }

    if args.offline {
        println!("⚠️ - Not attempting to perform online heartbeat check (--offline passed)")
    } else {
        match reqwest::Client::new()
                    .post(url.join("users/current/heartbeats").unwrap())
                    .bearer_auth(config.settings.api_key)
                    .body(format!(
                        "[{{\"type\":\"file\",\"time\":{},\"entity\":\"wakadoctor-test.txt\",\"language\":\"Text\"}}]",
                        time::UtcDateTime::now().unix_timestamp()
                    ))
                    .header("Content-Type", "application/json")
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await {
            Ok(_) => {
                println!("✅ - Got successful status code! {host} is configured correctly.")
            },
            Err(e) => {
                if e.is_timeout() {
                    println!(
                        "❌ - Server timeout after 10 seconds. {host} is NOT configured correctly."
                    );
                } else {
                    println!(
                        "❌ - Got error status code ({}). {host} is NOT configured correctly.",
                        e.status()
                            .map(|v| v.as_str().to_string())
                            .unwrap_or("no status code provided".to_string())
                    );
                }
                return;
            },
        };
    }
    
    println!("✅ - {host} is configured correctly!");
}
