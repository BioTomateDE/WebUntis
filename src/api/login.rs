use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::{
    Url,
    blocking::{Client, Response},
    cookie::Jar,
};
use serde::Serialize;

use crate::api::ApiClient;

#[derive(Serialize)]
struct AuthRequest<'a> {
    j_username: &'a str,
    j_password: &'a str,
}

pub fn login(username: &str, password: &str, school: &str) -> Result<ApiClient> {
    let base_url: String = format!("https://{school}.webuntis.com/WebUntis/");
    let base_url =
        Url::parse(&base_url).with_context(|| format!("Could not parse URL {base_url:?}"))?;

    let jar = Arc::new(Jar::default());

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .cookie_provider(Arc::clone(&jar))
        .build()?;

    let url = base_url.join("j_spring_security_check")?;
    let body = AuthRequest {
        j_username: username,
        j_password: password,
    };

    let resp: Response = client
        .post(url.clone())
        .header("Content-Type", "application/x-www-form-urlencoded") // lol
        .header("Accept", "application/json")
        .form(&body)
        .send()
        .context("Could not send request to j_spring_security_check")?;

    super::handle_response(resp)?;

    let url = base_url.join("api/token/new")?;
    let resp: Response = client
        .get(url.clone())
        .send()
        .context("Could not send request to token/new")?;
    let token: String =
        super::handle_response(resp).context("Bad response for token generation request")?;

    ApiClient::new(client, token, school)
}
