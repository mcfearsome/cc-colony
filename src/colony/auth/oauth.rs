use std::io::{Read, Write};
use std::net::TcpListener;
use serde::{Deserialize, Serialize};
use crate::error::{ColonyError, ColonyResult};

pub struct OAuthFlow {
    client_id: String,
    redirect_uri: String,
    auth_url: String,
    token_url: String,
}

impl OAuthFlow {
    pub fn new() -> Self {
        Self {
            client_id: "colony-cli".to_string(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            auth_url: "https://claude.ai/oauth/authorize".to_string(),
            token_url: "https://api.anthropic.com/oauth/token".to_string(),
        }
    }

    /// Start OAuth flow and open browser
    pub async fn authenticate(&self) -> ColonyResult<OAuthToken> {
        // 1. Start local HTTP server on port 8888
        let listener = TcpListener::bind("127.0.0.1:8888")
            .map_err(|e| ColonyError::Auth(format!("Failed to start local server: {}", e)))?;

        // 2. Generate PKCE challenge
        let (code_verifier, code_challenge) = generate_pkce_challenge();

        // 3. Generate state for CSRF protection
        let state = generate_random_state();

        // 4. Build authorization URL
        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&code_challenge={}&code_challenge_method=S256&scope=openid%20profile%20api&state={}",
            self.auth_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            code_challenge,
            state
        );

        // 5. Open browser
        println!("🔐 Opening browser for authentication...");
        println!("📋 If browser doesn't open, visit: {}", auth_url);
        println!();

        if let Err(e) = open::that(&auth_url) {
            eprintln!("⚠️  Could not open browser automatically: {}", e);
            println!("Please open the URL above in your browser manually.");
        }

        // 6. Wait for callback
        let (auth_code, returned_state) = self.wait_for_callback(listener)?;

        // 7. Verify state matches
        if returned_state != state {
            return Err(ColonyError::Auth("State mismatch - possible CSRF attack".to_string()));
        }

        // 8. Exchange code for token
        let token = self.exchange_code_for_token(&auth_code, &code_verifier).await?;

        Ok(token)
    }

    fn wait_for_callback(&self, listener: TcpListener) -> ColonyResult<(String, String)> {
        println!("⏳ Waiting for authentication...");
        println!("   (Check your browser)");
        println!();

        let (mut stream, _) = listener.accept()
            .map_err(|e| ColonyError::Auth(format!("Failed to accept connection: {}", e)))?;

        let mut buffer = [0; 2048];
        let bytes_read = stream.read(&mut buffer)
            .map_err(|e| ColonyError::Auth(format!("Failed to read request: {}", e)))?;

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);

        // Parse authorization code from query params
        let code = extract_query_param(&request, "code")
            .ok_or_else(|| {
                let error = extract_query_param(&request, "error");
                let error_desc = extract_query_param(&request, "error_description");

                if let Some(err) = error {
                    ColonyError::Auth(format!(
                        "Authentication failed: {} - {}",
                        err,
                        error_desc.unwrap_or_else(|| "No description".to_string())
                    ))
                } else {
                    ColonyError::Auth("No authorization code received".to_string())
                }
            })?;

        let state = extract_query_param(&request, "state").unwrap_or_default();

        // Send success response
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
            <!DOCTYPE html>\
            <html><head><title>Colony Authentication</title>\
            <style>\
                body { font-family: system-ui, -apple-system, sans-serif; \
                       display: flex; align-items: center; justify-content: center; \
                       height: 100vh; margin: 0; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); }\
                .card { background: white; padding: 3rem; border-radius: 1rem; \
                        box-shadow: 0 20px 60px rgba(0,0,0,0.3); text-align: center; max-width: 400px; }\
                .success { color: #10b981; font-size: 4rem; margin: 0; }\
                h1 { color: #1f2937; margin: 1rem 0; }\
                p { color: #6b7280; line-height: 1.6; }\
            </style></head>\
            <body>\
            <div class='card'>\
                <div class='success'>✅</div>\
                <h1>Authentication Successful!</h1>\
                <p>You can now close this window and return to your terminal.</p>\
                <p style='margin-top: 2rem; font-size: 0.875rem; color: #9ca3af;'>Colony CLI</p>\
            </div>\
            </body></html>";

        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();

        Ok((code, state))
    }

    async fn exchange_code_for_token(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> ColonyResult<OAuthToken> {
        let client = reqwest::Client::new();

        println!("🔄 Exchanging authorization code for token...");

        let response = client
            .post(&self.token_url)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.redirect_uri),
                ("client_id", &self.client_id),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await
            .map_err(|e| ColonyError::Auth(format!("Token exchange request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ColonyError::Auth(format!(
                "Token exchange failed ({}): {}",
                status,
                error_text
            )));
        }

        let mut token: OAuthToken = response.json().await
            .map_err(|e| ColonyError::Auth(format!("Failed to parse token response: {}", e)))?;

        // Calculate expiration time
        token.expires_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + token.expires_in
        );

        Ok(token)
    }

    /// Refresh an expired token
    pub async fn refresh_token(&self, refresh_token: &str) -> ColonyResult<OAuthToken> {
        let client = reqwest::Client::new();

        println!("🔄 Refreshing authentication token...");

        let response = client
            .post(&self.token_url)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &self.client_id),
            ])
            .send()
            .await
            .map_err(|e| ColonyError::Auth(format!("Token refresh failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ColonyError::Auth(format!(
                "Token refresh failed: {}",
                response.text().await.unwrap_or_default()
            )));
        }

        let mut token: OAuthToken = response.json().await?;

        // Calculate expiration time
        token.expires_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + token.expires_in
        );

        Ok(token)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

impl OAuthToken {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Consider expired if within 5 minutes of expiration
            now >= expires_at.saturating_sub(300)
        } else {
            false
        }
    }
}

fn generate_pkce_challenge() -> (String, String) {
    use base64::{engine::general_purpose, Engine as _};
    use rand::Rng;
    use sha2::{Digest, Sha256};

    // Generate code verifier (43-128 characters)
    let verifier: String = (0..64)
        .map(|_| {
            let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
            chars.chars().nth(rand::thread_rng().gen_range(0..chars.len())).unwrap()
        })
        .collect();

    // Generate code challenge (SHA256 hash)
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

    (verifier, challenge)
}

fn generate_random_state() -> String {
    use rand::Rng;

    (0..32)
        .map(|_| {
            let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            chars.chars().nth(rand::thread_rng().gen_range(0..chars.len())).unwrap()
        })
        .collect()
}

fn extract_query_param(request: &str, param: &str) -> Option<String> {
    // Find the query string in the HTTP request
    let first_line = request.lines().next()?;
    let query_start = first_line.find('?')?;
    let query_end = first_line[query_start..].find(' ')?;
    let query_string = &first_line[query_start + 1..query_start + query_end];

    // Parse query parameters
    for pair in query_string.split('&') {
        let mut parts = pair.split('=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            if key == param {
                return Some(urlencoding::decode(value).ok()?.into_owned());
            }
        }
    }

    None
}
