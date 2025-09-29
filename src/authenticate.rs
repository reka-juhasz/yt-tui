use anyhow::{anyhow, Result};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RefreshToken,
    Scope, StandardTokenResponse, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use tiny_http::{Response, Server};
use url::Url;
use webbrowser;

pub type OAuthToken =
    StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>;

#[derive(Debug, Deserialize)]
pub struct Installed {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub installed: Installed,
}

pub fn read_credentials(path: &str) -> Result<Credentials> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let creds = serde_json::from_reader(reader)?;
    Ok(creds)
}

pub fn save_token(token: &OAuthToken) -> Result<()> {
    let json = serde_json::to_string_pretty(token)?;
    fs::write("token.json", json)?;
    Ok(())
}

pub fn load_token() -> Result<OAuthToken> {
    let data = fs::read_to_string("token.json")?;
    let token: OAuthToken = serde_json::from_str(&data)?;
    Ok(token)
}

pub async fn authenticate<F>(mut display_message: F) -> Result<OAuthToken>
where
    F: FnMut(&str),
{
    let creds = read_credentials("credentials.json")?;
    let client_id = ClientId::new(creds.installed.client_id.clone());
    let client_secret = ClientSecret::new(creds.installed.client_secret.clone());

    // Localhost redirect with port 8080
    let redirect_uri = "http://127.0.0.1:8080";
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?;
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?;

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?);

    // Try refreshing existing token if available
    if Path::new("token.json").exists() {
        let token = load_token()?;
        if let Some(refresh_token) = token.refresh_token() {
            let new_token = client
                .exchange_refresh_token(&RefreshToken::new(refresh_token.secret().to_string()))
                .request_async(async_http_client)
                .await?;
            save_token(&new_token)?;
            display_message("Token refreshed successfully.");
            return Ok(new_token);
        }
    }

    // Build authorization URL
    let (auth_url, _csrf_token) = client
        .authorize_url(|| CsrfToken::new_random())
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/youtube".to_string(),
        ))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .url();

    println!("Full authorization URL:\n{}", auth_url.as_str());

    // Open browser
    display_message("Opening your browser to authenticate...");
    webbrowser::open(auth_url.as_str())?;

    // Start local HTTP server to capture redirect
    let server = Server::http("127.0.0.1:8080")
        .map_err(|e| anyhow!("Failed to start local HTTP server: {}", e))?;
    display_message("Waiting for authentication response...");

    let request = server.recv()?; // blocks until redirect

    // Extract code from query parameters
    let url = Url::parse(&format!("http://localhost{}", request.url()))?;
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| anyhow!("No 'code' parameter found in redirect"))?;

    // Respond to the browser
    let response = Response::from_string(
        "Authentication complete! You can close this browser tab and return to the app.",
    );
    request.respond(response)?;

    // Exchange code for token
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await?;
    save_token(&token)?;
    display_message("Authentication complete!");

    Ok(token)
}
