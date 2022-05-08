mod conoha;

use conoha::{
    dns::{Domain, Record},
    ConohaClient,
};
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
    domain.eq(full_domain)
        || regex::Regex::new(format!(r"\.{}$", domain).as_str())
            .unwrap()
            .is_match(full_domain)
}

fn get_domain_without_dot(domain_with_dot: &str) -> String {
    let mut domain_without_dot = domain_with_dot.to_string();
    domain_without_dot.pop();
    domain_without_dot
}

fn search_correspond_domain_id(
    domain_list: &conoha::dns::DomainListResponse,
    full_domain: &str,
) -> Result<uuid::Uuid, String> {
    for domain in &domain_list.domains {
        let domain_without_dot = get_domain_without_dot(domain.name.as_str());

        if is_same_domain(full_domain, domain_without_dot.as_str()) {
            return Ok(domain.id.unwrap());
        }
    }

    Err(format!(
        "domain item corresponds {} was not found",
        full_domain
    ))
}

async fn cleanup(
    client: &ConohaClient,
    domain_id: &uuid::Uuid,
    target_record_name: &str,
) -> Result<(), Box<dyn error::Error>> {
    let r: conoha::dns::RecordListResponse = client
        .get(
            format!(
                "https://dns-service.tyo1.conoha.io/v1/domains/{}/records",
                domain_id.as_hyphenated().to_string()
            )
            .as_str(),
        )
        .await?;

    for record in r.records {
        if record.name.eq(target_record_name) {
            println!("record \"{}\" will be deleted...", record.name);

            client
                .delete(
                    format!(
                        "https://dns-service.tyo1.conoha.io/v1/domains/{}/records/{}",
                        domain_id.as_hyphenated().to_string(),
                        record.id.unwrap().as_hyphenated().to_string(),
                    )
                    .as_str(),
                )
                .await?;
            println!("finished");
        }
    }

    Ok(())
}

async fn auth(
    client: &ConohaClient,
    domain_id: &uuid::Uuid,
    target_record_name: &str,
    validation_token: &str,
) -> Result<(), Box<dyn error::Error>> {
    let validation_record = Record {
        name: target_record_name.to_string(),
        record_type: "TXT".to_string(),
        data: validation_token.to_string(),
        ttl: Some(60),
        ..Default::default()
    };

    println!("create {}", serde_json::to_string(&validation_record)?);

    let res: serde_json::Value = client
        .post(
            format!(
                "https://dns-service.tyo1.conoha.io/v1/domains/{}/records",
                domain_id.as_hyphenated().to_string(),
            )
            .as_str(),
            &validation_record,
        )
        .await?;

    println!("created {}", res.to_string());

    Ok(())
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

    let config: AppConfig = confy::load_path("config.toml")?;

    let client = ConohaClient::new()?
        .load_token(
            config.conoha_id_config.identity_endpoint.as_str(),
            config.conoha_id_config.username.as_str(),
            config.conoha_id_config.password.as_str(),
            config.conoha_id_config.tenant_id.as_str(),
        )
        .await?;

    let record_target = format!("_acme-challenge.{}.", full_domain);

    let domain_list: conoha::dns::DomainListResponse = client
        .get("https://dns-service.tyo1.conoha.io/v1/domains")
        .await?;

    let domain_item_id = search_correspond_domain_id(&domain_list, full_domain.as_str())?;

    if clenaup_mode {
        cleanup(&client, &domain_item_id, record_target.as_str()).await?;
    } else {
        auth(
            &client,
            &domain_item_id,
            record_target.as_str(),
            validation_token.as_str(),
        )
        .await?;
    }

    Ok(())
}
