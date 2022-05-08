use actix_web::http::Method;
use awc::{Client, ClientRequest, Connector};
use chrono::{DateTime, Utc};
use openssl::ssl::{SslConnector, SslMethod};
use serde::{de::DeserializeOwned, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::error;

pub mod dns;
pub mod identity;

#[derive(Serialize, Deserialize)]
pub struct ConohaToken {
    #[serde(rename = "id")]
    pub token_id: String,

    pub expires: DateTime<Utc>,
}

pub struct ConohaClient {
    token: Option<ConohaToken>,
    web_client: Client,
}

impl ConohaClient {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        Ok(ConohaClient {
            token: None,
            web_client: build_web_client(&build_ssl_connector()?),
        })
    }

    pub async fn load_token(
        self,
        identity_endpoint: &str,
        username: &str,
        password: &str,
        tenant_id: &str,
    ) -> Result<ConohaClient, Box<dyn error::Error>> {
        let token = identity::get_token(
            &self.web_client,
            identity_endpoint,
            username,
            password,
            tenant_id,
        )
        .await?;

        Ok(ConohaClient {
            token: Some(token),
            web_client: self.web_client,
        })
    }

    fn token_request(
        &self,
        method: awc::http::Method,
        url: &str,
    ) -> Result<ClientRequest, Box<dyn error::Error>> {
        let token = self
            .token
            .as_ref()
            .ok_or("token not loaded")?
            .token_id
            .clone();

        let request = self
            .web_client
            .request(method, url)
            .append_header(("X-Auth-Token", token));

        Ok(request)
    }

    pub async fn get<TRet: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<TRet, Box<dyn error::Error>> {
        let request = self.token_request(Method::GET, url)?;
        let mut res = request.send().await?;
        Ok(res.json::<TRet>().await?)
    }

    pub async fn post<TRet: DeserializeOwned, TParam: Serialize>(
        &self,
        url: &str,
        body: &TParam,
    ) -> Result<TRet, Box<dyn error::Error>> {
        let request = self.token_request(Method::POST, url)?;
        let mut res = request.send_json(body).await?;
        Ok(res.json::<TRet>().await?)
    }

    pub async fn delete(
        &self,
        url: &str,
    ) -> Result<(), Box<dyn error::Error>> {
        let request = self.token_request(Method::DELETE, url)?;
        request.send().await?;
        Ok(())
    }

    pub async fn put<TRet: DeserializeOwned, TParam: Serialize>(
        &self,
        url: &str,
        body: &TParam,
    ) -> Result<TRet, Box<dyn error::Error>> {
        let request = self.token_request(Method::PUT, url)?;
        let mut res = request.send_json(body).await?;
        Ok(res.json::<TRet>().await?)
    }
}

fn build_ssl_connector() -> Result<SslConnector, Box<dyn error::Error>> {
    let mut builder = SslConnector::builder(SslMethod::tls())?;
    builder.set_ca_file("cacert.pem")?;

    Ok(builder.build())
}

fn build_web_client(ssl_connector: &SslConnector) -> Client {
    let connector = Connector::new().openssl(ssl_connector.clone());

    Client::builder().connector(connector).finish()
}
