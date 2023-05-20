use std::{fs, process::exit, sync::Arc};

use serde::{Serialize, Deserialize};
use ureq::serde_json::Value;

#[derive(Deserialize)]
struct IpInfo {
    ip: String,
}

#[derive(Serialize)]
struct Network {
    network: String,
}

#[derive(Serialize)]
struct RequestJson {
    name: String,
    networks: Vec<Network>,
}

fn main() -> Result<(), ureq::Error> {
    let config = fs::read_to_string("./config.conf").expect("Missing config.conf file");
    let mut vars = config.lines();

    let acc_id = vars.next().expect("Missing account ID argument");
    let location_uuid = vars.next().expect("Missing location UUID");
    let location_name = vars.next().expect("Missing location name");
    let acc_email = vars.next().expect("Missing account email");
    let acc_auth_key = vars.next().expect("Missing account auth key");

    let agent = ureq::AgentBuilder::new()
        .tls_connector(Arc::new(
            native_tls::TlsConnector::new()
                .expect("Failed to init native tls, this is needed for security."),
        ))
        .build();

    let ip = agent
        .get("https://ipv4.teams.cloudflare.com")
        .call()?
        .into_json::<IpInfo>()?
        .ip;

    let resp = agent.put(&format!("https://api.cloudflare.com/client/v4/accounts/{acc_id}/gateway/locations/{location_uuid}"))
        .set("X-Auth-Email", acc_email)
        .set("X-Auth-Key", acc_auth_key)
        .send_json(RequestJson {
            name: location_name.to_string(),
            networks: vec![
                Network {
                    network: ip.clone() + "/32",
                }
            ]
        })
        .map_err(|e| match e {
            Error::Status(status, resp) => {
                eprintln!("{status} error: {}", resp.into_string().expect("Can't convert body to string"));
                exit(1)
            }
            _ => e
        })?
        .into_json::<Value>()?;

    if !resp["success"] {
        eprintln!("Failed to update gateway origin\n{}", resp.to_string());
        exit(1);
    }
    println!("Updated `{location_name}` to `{ip}`");

    Ok(())
}
