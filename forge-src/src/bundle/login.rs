use crate::api::ApiClient;
use crate::config::ForgeConfig;
use crate::error::ForgeResult;
use crate::utils;

pub async fn run() -> ForgeResult<()> {
    utils::header("Login to useforge.cc");

    let email = utils::input("Email").ok_or("Email is required")?;

    let password = utils::password("Password").ok_or("Password is required")?;

    let spinner = utils::spinner("Authenticating...");

    let client = ApiClient::new()?;
    let response = client.login(email, password).await?;

    spinner.finish_and_clear();

    // Save token to config
    let mut config = ForgeConfig::load()?;
    config.api_token = Some(response.token);
    config.save()?;

    utils::success(&format!("Logged in as {}", response.user.email));

    if let Some(name) = response.user.name {
        utils::info(&format!("Welcome, {}!", name));
    }

    Ok(())
}
