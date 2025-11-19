use crate::colony::config::ColonyConfig;
use crate::colony::controller::ColonyController;
use crate::colony::relay::{RelayClient, RelayConfig};
use crate::error::{ColonyError, ColonyResult};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

/// Relay connection configuration stored in .colony/relay.json
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RelayConnectionConfig {
    url: String,
    colony_id: String,
    auth_token: String,
}

/// Get path to relay config file
fn relay_config_path() -> PathBuf {
    Path::new(".colony").join("relay.json")
}

/// Connect to relay service
pub async fn connect(url: Option<String>, token: Option<String>) -> ColonyResult<()> {
    println!("{}", "🔗 Connecting to Colony Relay Service".bold());
    println!();

    // Load colony config
    let config_path = Path::new("colony.yml");
    if !config_path.exists() {
        return Err(ColonyError::Colony(
            "No colony.yml found. Initialize a colony first with 'colony init'".to_string(),
        ));
    }

    let colony_config = ColonyConfig::load(config_path)?;
    let colony_name = colony_config
        .name
        .clone()
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Get relay URL
    let relay_url = url.unwrap_or_else(|| {
        std::env::var("COLONY_RELAY_URL")
            .unwrap_or_else(|_| "wss://api.colony.sh".to_string())
    });

    // Get or prompt for auth token
    let auth_token = if let Some(token) = token {
        token
    } else if let Ok(token) = std::env::var("COLONY_RELAY_TOKEN") {
        token
    } else {
        // Prompt user for token
        println!("To connect your colony to the relay service, you need an authentication token.");
        println!("Get your token from: https://app.colony.sh/settings/tokens");
        println!();

        use dialoguer::Input;
        Input::<String>::new()
            .with_prompt("Authentication token")
            .interact_text()
            .map_err(|e| ColonyError::Colony(format!("Failed to read input: {}", e)))?
    };

    // Generate colony ID
    let colony_id = format!("{}-{}", colony_name, uuid::Uuid::new_v4().to_string()[..8].to_string());

    // Save relay config
    let relay_config_data = RelayConnectionConfig {
        url: relay_url.clone(),
        colony_id: colony_id.clone(),
        auth_token: auth_token.clone(),
    };

    fs::create_dir_all(".colony")?;
    let relay_config_json = serde_json::to_string_pretty(&relay_config_data)?;
    fs::write(relay_config_path(), relay_config_json)?;

    println!("✓ Relay configuration saved");
    println!();

    // Create relay client
    let relay_config = RelayConfig {
        url: relay_url,
        colony_id: colony_id.clone(),
        auth_token,
    };

    let controller = ColonyController::new(colony_config)?;
    let colony_root = std::env::current_dir()?;
    let client = RelayClient::new(relay_config, controller, colony_root);

    println!("{}", "Colony connected to relay service!".green().bold());
    println!("Colony ID: {}", colony_id.cyan());
    println!();
    println!("You can now control your colony from:");
    println!("  🌐 https://app.colony.sh");
    println!("  📱 Colony mobile app");
    println!();
    println!("Press Ctrl+C to disconnect");
    println!();

    // Connect and run (blocks until disconnected)
    client.connect().await?;

    Ok(())
}

/// Show relay connection status
pub async fn status() -> ColonyResult<()> {
    let config_path = relay_config_path();

    if !config_path.exists() {
        println!("{}", "Not connected to relay service".yellow());
        println!();
        println!("Connect with: colony relay connect");
        return Ok(());
    }

    let config_json = fs::read_to_string(&config_path)?;
    let config: RelayConnectionConfig = serde_json::from_str(&config_json)?;

    println!("{}", "Relay Connection Status".bold());
    println!();
    println!("  Status:     {}", "Connected".green());
    println!("  Relay URL:  {}", config.url);
    println!("  Colony ID:  {}", config.colony_id.cyan());
    println!();
    println!("Web Dashboard: https://app.colony.sh");
    println!();
    println!("To disconnect: colony relay disconnect");

    Ok(())
}

/// Disconnect from relay service
pub async fn disconnect() -> ColonyResult<()> {
    let config_path = relay_config_path();

    if !config_path.exists() {
        println!("{}", "Not connected to relay service".yellow());
        return Ok(());
    }

    fs::remove_file(&config_path)?;

    println!("{}", "✓ Disconnected from relay service".green());
    println!();
    println!("To reconnect: colony relay connect");

    Ok(())
}
