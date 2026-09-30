// main.rs
// rustyflare - A simple Cloudflare DDNS client written in Rust
//
// ENV:
// - CF_API_TOKEN: Your Cloudflare API token
// - CF_DOMAIN: The domain name to update (e.g., sub.example.com)
//
// --prrar

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process::ExitCode;
use std::time::Duration;
use ureq::Agent;

// Constants for the URLs used in the application
const GET_IP_URL: &str = "https://ipv4.icanhazip.com";
const API_URL: &str = "https://api.cloudflare.com/client/v4";

// Structs for deserializing JSON responses from the Cloudflare API
#[derive(Debug, Deserialize)]
struct Zone {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Record {
    id: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct NewRecord {
    content: String,
}

// Generic response struct to handle API responses with a result field
#[derive(Debug, Deserialize)]
struct Response<T> {
    result: Vec<T>,
}

struct Cloudflare {
    agent: Agent,
    header: String,
}

impl Cloudflare {
    fn new(agent: Agent, token: &str) -> Self {
        let header = format!("Bearer {token}");

        Self { agent, header }
    }

    fn get_zone_id(&self, domain: &str) -> Result<String, Box<dyn Error>> {
        let zones: Response<Zone> = self
            .agent
            .get(format!("{API_URL}/zones?per_page=50&status=active"))
            .header("Authorization", &self.header)
            .call()
            .map_err(|e| format!("listing zones: {e}"))?
            .body_mut()
            .read_json()
            .map_err(|e| format!("listing zones: {e}"))?;
        let zone_id = zones
            .result
            .into_iter()
            .filter(|z| domain == z.name || domain.ends_with(&format!(".{}", z.name)))
            .max_by_key(|z| z.name.len())
            .ok_or("zone for the domain was not found")?
            .id;
        Ok(zone_id)
    }

    fn get_record(&self, zone_id: &str, domain: &str) -> Result<Record, Box<dyn Error>> {
        let records: Response<Record> = self
            .agent
            .get(format!(
                "{API_URL}/zones/{zone_id}/dns_records?type=A&name.exact={domain}"
            ))
            .header("Authorization", &self.header)
            .call()
            .map_err(|e| format!("listing records: {e}"))?
            .body_mut()
            .read_json()
            .map_err(|e| format!("listing records: {e}"))?;
        if records.result.len() > 1 {
            return Err("more than one record for the domain was found".into());
        }
        let record = records
            .result
            .into_iter()
            .next()
            .ok_or("record for the domain was not found")?;
        Ok(record)
    }

    fn patch_record(
        &self,
        zone_id: &str,
        record_id: &str,
        new_ip: &str,
    ) -> Result<(), Box<dyn Error>> {
        let new_record = NewRecord {
            content: new_ip.to_string(),
        };
        self.agent
            .patch(format!("{API_URL}/zones/{zone_id}/dns_records/{record_id}"))
            .header("Authorization", &self.header)
            .send_json(&new_record)
            .map_err(|e| format!("patching record: {e}"))?;
        Ok(())
    }
}

fn get_ip(agent: &Agent) -> Result<String, Box<dyn Error>> {
    let body = agent
        .get(GET_IP_URL)
        .call()
        .map_err(|e| format!("fetching public IP: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("fetching public IP: {e}"))?;
    let ip: std::net::Ipv4Addr = body
        .trim()
        .parse()
        .map_err(|e| format!("invalid IP from {GET_IP_URL}: {e}"))?;
    Ok(ip.to_string())
}

fn from_env() -> Result<(String, String), Box<dyn Error>> {
    let token = std::env::var("CF_API_TOKEN").map_err(|_| "CF_API_TOKEN not defined.")?;
    let domain = std::env::var("CF_DOMAIN")
        .map_err(|_| "CF_DOMAIN not defined.")?
        .trim_end_matches('.')
        .to_lowercase();
    Ok((token, domain))
}

fn run() -> Result<(), Box<dyn Error>> {
    let (token, domain) = from_env()?;
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .into();
    let ip = get_ip(&agent)?;
    let cf = Cloudflare::new(agent, &token);
    let zone_id = cf.get_zone_id(&domain)?;
    let record = cf.get_record(&zone_id, &domain)?;
    if ip == record.content {
        println!("[{domain}] IP address {ip} unchanged, no update needed.");
        return Ok(());
    }
    cf.patch_record(&zone_id, &record.id, &ip)?;
    println!("[{domain}] Updated IP: {ip}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
