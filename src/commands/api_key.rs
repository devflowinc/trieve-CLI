use trieve_client::{apis::configuration::Configuration, models::SetUserApiKeyRequest};

use crate::ApiKeyData;

use super::configure::TrieveConfiguration;

pub async fn generate_api_key(
    settings: TrieveConfiguration,
    api_key_data: ApiKeyData,
) -> Result<(), Box<dyn std::error::Error>> {
    if settings.organization_id.to_string().is_empty() || settings.api_key.is_empty() {
        eprintln!("Please login to the Trieve CLI with your credentials. Run `trieve login` to get started.");
        std::process::exit(1);
    }

    let name = if api_key_data.name.is_none() {
        inquire::Text::new("Enter a name for the API Key:")
            .with_help_message("This name will help you identify the API Key in the future.")
            .prompt()
            .unwrap()
    } else {
        api_key_data.name.unwrap()
    };

    let role = if api_key_data.role.is_none() {
        inquire::Select::new(
            "Select a role for the API Key:",
            vec!["Read + Write", "Read"],
        )
        .prompt()
        .unwrap()
        .to_string()
    } else {
        api_key_data.role.unwrap()
    };

    let role_num = match role {
        r if r == "Read + Write" => 1,
        r if r == "Read" => 0,
        _ => {
            eprintln!("Invalid role: {}", role);
            std::process::exit(1);
        }
    };

    let config = Configuration {
        base_path: settings.api_url.clone(),
        api_key: Some(trieve_client::apis::configuration::ApiKey {
            prefix: None,
            key: settings.api_key.clone(),
        }),
        ..Default::default()
    };

    let data = SetUserApiKeyRequest {
        name: name.clone(),
        dataset_ids: None,
        organization_ids: None,
        scopes: None,
        role: role_num,
    };

    let user = trieve_client::apis::user_api::set_user_api_key(&config, data)
        .await
        .map_err(|e| {
            eprintln!("Error generating API Key: {:?}", e);
            std::process::exit(1);
        });

    match user {
        Ok(api_key) => {
            println!("\nAPI Key generated successfully!\n");
            println!("Name: {}", name);
            println!("API Key: {}", api_key.api_key);
        }
        Err(_) => {
            eprintln!("Error generating API Key.");
            std::process::exit(1);
        }
    }

    Ok(())
}
