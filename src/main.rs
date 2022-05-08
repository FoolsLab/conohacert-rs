mod conoha;

use conoha::{dns::Record, ConohaClient};
use serde_derive::{Deserialize, Serialize};
use std::{env, error};

#[derive(Default, Serialize, Deserialize)]
struct AppConfig {
    conoha_id_config: ConohaIdConfig,
}

#[derive(Default, Serialize, Deserialize)]
struct ConohaIdConfig {
    identity_endpoint: String,
    username: String,
    password: String,
    tenant_id: String,
}

fn is_same_domain(full_domain: &str, domain: &str) -> bool {
    if domain.eq(full_domain) {
        true
    } else if regex::Regex::new(format!(r"\.{}$", domain).as_str())
        .unwrap()
        .is_match(full_domain)
    {
        true
    } else {
        false
    }
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 4);

    let clenaup_mode = args[1].eq("clean");

    // let full_domain = env::var("CERTBOT_DOMAIN")?;
    // let validation_token = env::var("CERTBOT_VALIDATION")?;
    let full_domain = args[2].clone();
    let validation_token = args[3].clone();

    let record_target = format!("_acme-challenge.{}.", full_domain);

    let config: AppConfig = confy::load_path("config.toml")?;

    let client = ConohaClient::new()?
        .load_token(
            config.conoha_id_config.identity_endpoint.as_str(),
            config.conoha_id_config.username.as_str(),
            config.conoha_id_config.password.as_str(),
            config.conoha_id_config.tenant_id.as_str(),
        )
        .await?;

    let v: conoha::dns::DomainListResponse = client
        .get("https://dns-service.tyo1.conoha.io/v1/domains")
        .await?;

    for domain in v.domains {
        let mut domain_without_dot = domain.name.clone();
        domain_without_dot.pop();
        let domain_without_dot = domain_without_dot;

        if !is_same_domain(full_domain.as_str(), domain_without_dot.as_str()) {
            continue;
        }

        let id = domain.id.unwrap();

        if clenaup_mode {
            let r: conoha::dns::RecordListResponse = client
                .get(
                    format!(
                        "https://dns-service.tyo1.conoha.io/v1/domains/{}/records",
                        id.as_hyphenated().to_string()
                    )
                    .as_str(),
                )
                .await?;

            for record in r.records {
                if record.name.eq(record_target.as_str()) {
                    println!("record \"{}\" will be deleted...", record.name);

                    client
                        .delete(
                            format!(
                                "https://dns-service.tyo1.conoha.io/v1/domains/{}/records/{}",
                                id.as_hyphenated().to_string(),
                                record.id.unwrap().as_hyphenated().to_string(),
                            )
                            .as_str(),
                        )
                        .await?;
                        
                    println!("finished");
                }
            }
        } else {
            let validation_record = Record {
                name: record_target.clone(),
                record_type: "TXT".to_string(),
                data: validation_token.clone(),
                ttl: Some(60),
                ..Default::default()
            };

            println!("create {}", serde_json::to_string(&validation_record)?);

            let res: serde_json::Value = client
                .post(
                    format!(
                        "https://dns-service.tyo1.conoha.io/v1/domains/{}/records",
                        id.as_hyphenated().to_string(),
                    )
                    .as_str(),
                    &validation_record,
                )
                .await?;

            println!("{}", res.to_string());
            break;
        }

        break;
    }

    Ok(())
}
