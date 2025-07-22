use anyhow::{Context, Result};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RefreshToken,
    Scope, StandardTokenResponse, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use url::Url;
use webbrowser;
pub type OAuthToken =
    StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>;

#[derive(Debug, Deserialize)]
struct Installed {
    client_id: String,
    client_secret: String,
    redirect_uris: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    installed: Installed,
}

pub fn read_credentials(path: &str) -> Result<Credentials> {
    let file = File::open(path)?; //opens file
    let reader = BufReader::new(file); //creates buffer
    let creds = serde_json::from_reader(reader)?; //opens buffer as a variable
    Ok(creds)
}

pub fn save_token(token: &OAuthToken) -> Result<()> {
    let json = serde_json::to_string_pretty(token)?; //takes the oauth token and converts
                                                     //it into json
    fs::write("token.json", json)?; //writes out
    Ok(())
}

pub fn load_token() -> Result<OAuthToken> {
    let data = fs::read_to_string("token.json")?;
    let token: OAuthToken = serde_json::from_str(&data)?;
    Ok(token)
}

pub async fn authenticate<F>(mut display_message: F) -> Result<OAuthToken>
where
    F: FnMut(&str), //closure taking slices
{
    let creds = read_credentials("credentials.json")?; //reads in user creds

    let client_id = ClientId::new(creds.installed.client_id.clone()); //clones client id
    let client_secret = ClientSecret::new(creds.installed.client_secret.clone()); //clones client secret

    let redirect_uri = creds
        .installed
        .redirect_uris
        .get(0)
        .context("No redirect URI found")?
        .to_string(); //creating redirect uri

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?; //auth
                                                                                              //and
                                                                                              //token
                                                                                              //urls
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?;

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?); //builds the oauth client

    if Path::new("token.json").exists() {
        let token = load_token()?;
        if let Some(refresh_token) = token.refresh_token() {
            let new_token = client
                .exchange_refresh_token(&RefreshToken::new(refresh_token.secret().to_string()))
                .request_async(async_http_client)
                .await?;

            save_token(&new_token)?;
            display_message("Token refreshed successfully.");
            return Ok(new_token); //if there's already a token, it tries refreshing
        }
    }
    //only getting here if no access token or no refresh token found
    let (mut auth_url, _csrf_token) = client
        .authorize_url(|| CsrfToken::new_random()) //cross site
        //request
        //forgery
        //token
        //generated
        //if i feel
        //like i need
        //to use it
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/youtube".to_string(), //scope
        ))
        .url(); //builds authorization url

    auth_url.set_query(Some(&format!(
        "{}&access_type=offline&prompt=consent",
        auth_url.query().unwrap_or("")
    )));

    display_message("Please follow the pop-up tab in your browser:");
    display_message(auth_url.as_str());
    webbrowser::open(auth_url.as_str());

    display_message("Paste the full URL you received:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Parse the URL and extract the 'code' parameter
    let input = input.trim();
    let parsed_url = Url::parse(input)?;
    let code = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string())
        .ok_or_else(|| anyhow::anyhow!("No 'code' parameter found in the URL"))?;

    // Use the extracted code to get the token
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await?;
    save_token(&token)?;
    display_message(
        "Authentication complete, you may need to restart the tui for changes to take effect!",
    );
    Ok(token)
}
