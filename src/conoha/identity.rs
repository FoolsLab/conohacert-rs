use awc::Client;
use chrono::Utc;
use serde_json::json;
use std::{
    error::{self, Error},
    fs::File,
    io::{Read, Write}, fmt::{Display, self},
};

use super::ConohaToken;

#[derive(Debug)]
enum TokenCacheLoadError {
    TokenExpirationError,
}

impl Display for TokenCacheLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenCacheLoadError::TokenExpirationError => write!(f, "token expired"),
        }
    }
}

impl Error for TokenCacheLoadError {}

pub async fn get_token(
    client: &Client,
    identity_endpoint: &str,
    username: &str,
    password: &str,
    tenant_id: &str,
) -> Result<ConohaToken, Box<dyn error::Error>> {
    let cache_file_path = "tokencache.toml";

    match get_cached_token(cache_file_path) {
        Ok(token) => {
            println!("token loaded from cache");

            Ok(token)
        }
        Err(_) => {
            let token =
                get_new_token(&client, identity_endpoint, username, password, tenant_id).await?;
            cache_token(cache_file_path, &token)?;

            println!("token renewed and cached");

            Ok(token)
        }
    }
}

fn cache_token(path: &str, token: &ConohaToken) -> Result<(), Box<dyn error::Error>> {
    let str = toml::to_string(token)?;

    let mut f = File::create(path)?;
    f.write_all(str.as_bytes())?;

    Ok(())
}

fn get_cached_token(path: &str) -> Result<ConohaToken, Box<dyn error::Error>> {
    let mut f = File::open(path)?;
    let mut txt = String::new();
    f.read_to_string(&mut txt)?;

    let token: ConohaToken = toml::from_str(txt.as_str())?;

    if token.expires.cmp(&Utc::now()).is_lt() {
        Err(Box::new(TokenCacheLoadError::TokenExpirationError))
    } else {
        Ok(token)
    }
}

async fn get_new_token(
    client: &Client,
    identity_endpoint: &str,
    username: &str,
    password: &str,
    tenant_id: &str,
) -> Result<ConohaToken, Box<dyn error::Error>> {
    let query_body = json!({
        "auth": {
            "passwordCredentials": {
                "username": username,
                "password": password
            },
            "tenantId": tenant_id
        }
    });

    let mut res = client
        .post(format!("{}/tokens", identity_endpoint))
        .send_json(&query_body)
        .await?;

    let res = res.json::<serde_json::Value>().await?;

    let token: ConohaToken = serde_json::from_value(res["access"]["token"].to_owned())?;

    Ok(token)
}
