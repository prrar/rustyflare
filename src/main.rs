// main.rs
// rustyflare - A simple Cloudflare DDNS client written in Rust
//
// ENV:
// - CF_API_TOKEN: Your Cloudflare API token
// - CF_DOMAIN: The domain name to update (e.g., sub.example.com)
// - CF_ZONE_ID: (Optional) The Cloudflare zone ID. If not provided, the program will
//               fetch all zones and search for the record.
//
// --prrar

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;
use ureq::Agent;

// Constants for the URLs used in the application
const GET_IP_URL: &str = "https://ipv4.icanhazip.com";
const API_URL: &str = "https://api.cloudflare.com/client/v4";

// Structs for deserializing JSON responses from the Cloudflare API
#[derive(Debug, Deserialize)]
struct Zone {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    id: String,
    name: String,
    content: String,
}

// Generic response struct to handle API responses with a result field
#[derive(Debug, Deserialize)]
struct Response<T> {
    result: Vec<T>,
}

// Function to retrieve all active zones
fn get_zones(agent: &Agent, token: &str) -> Result<Vec<Zone>, ureq::Error> {
    let header = format!("Bearer {token}");
    let url: String = format!("{API_URL}/zones?per_page=50&status=active");
    let response: Response<Zone> = agent.get(url)
        .header("Authorization", header)
        .call()?
        .body_mut()
        .read_json()?;

    Ok(response.result)
}

// Function to search for a specific DNS record by domain name
fn search_record(agent: &Agent, token: &str, zone_id: &str, domain: &str) -> Result<Vec<Record>, ureq::Error> {
    let header = format!("Bearer {token}");
    let url: String = format!("{API_URL}/zones/{zone_id}/dns_records?type=A&name.exact={domain}");
    let response: Response<Record> = agent.get(url)
        .header("Authorization", header)
        .call()?
        .body_mut()
        .read_json()?;

    Ok(response.result)
}

// Function to update a DNS record with a new IP address
fn update_record(agent: &Agent, token: &str, zone_id: &str, record: &Record, new_ip: &str) -> Result<(), ureq::Error> {
    let header = format!("Bearer {token}");
    let url: String = format!("{API_URL}/zones/{zone_id}/dns_records/{}",record.id);
    let new_record = Record {
        id: record.id.clone(),
        name: record.name.clone(),
        content: new_ip.to_string(),
    };
    agent.patch(url)
        .header("Authorization", header)
        .send_json(&new_record)?;

    Ok(())
}

// Function to get the current public IP address of the machine
fn get_ip(agent: &Agent) -> Result<String, ureq::Error> {
    let body = agent.get(GET_IP_URL)
        .call()?
        .body_mut()
        .read_to_string()?;
    Ok(body.trim().to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Retrieve the Cloudflare API token and domain from environment variables
    let token = std::env::var("CF_API_TOKEN")
        .map_err(|_| "CF_API_TOKEN not defined")?
        .trim()
        .to_string();
    if token.is_empty() {
        return Err("CF_API_TOKEN is empty".into());
    }

    let domain = std::env::var("CF_DOMAIN")
        .map_err(|_| "CF_DOMAIN not defined")?
        .trim()
        .trim_end_matches('.')
        .to_lowercase();
    if domain.is_empty() {
        return Err("CF_DOMAIN is empty".into());
    }

    // Create an HTTP agent with a global timeout of 10 seconds
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .into();

    // Retrieve the zone ID from the environment variable or fetch all zones if not provided
    let zones = match std::env::var("CF_ZONE_ID").unwrap_or_default().trim() {
        "" => get_zones(&agent, &token)?,
        id => vec![Zone { id: id.to_string() }],
    };

    // Iterate through all zones and search for the specified DNS record
    for zone in zones {
        // Search for the DNS record in the current zone
        let records = search_record(&agent, &token, &zone.id, &domain)?;
        if let Some(record) = records.first() {
            let ip = get_ip(&agent)?;
            // Check if the current IP address is the same as the record content
            if ip == record.content {
                println!("IP address ({ip}) is the same as the record content, no update needed.");
                return Ok(());
            }
            // Update the DNS record with the new IP address
            update_record(&agent, &token, &zone.id, record, &ip)?;
            println!("Updated record: {}: {} -> {}", record.name, record.content, ip);
            return Ok(());
        }
    }
    // If no matching record is found in any zone, return an error
    Err(format!("Record {} not found", domain).into())
}