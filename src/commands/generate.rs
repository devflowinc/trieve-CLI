use trieve_client::{
    apis::{configuration::Configuration, user_api::SetUserApiKeyParams},
    models::SetUserApiKeyRequest,
};

use super::configure::TrieveConfiguration;

pub async fn generate_api_key(
    settings: TrieveConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    if settings.organization_id.to_string().is_empty() || settings.api_key.is_empty() {
        eprintln!("Please configure the Trieve CLI with your credentials. Run `trieve configure` to get started.");
        std::process::exit(1);
    }

    let name = inquire::Text::new("Enter a name for the API Key:")
        .with_help_message("This name will help you identify the API Key in the future.")
        .prompt()
        .unwrap();

    let role = inquire::Select::new(
        "Select a role for the API Key:",
        vec!["Read + Write", "Read"],
    )
    .with_help_message("Read + Write: Can read and write data. Read: Can only read data.")
    .prompt()
    .unwrap();

    let role_num = match role {
        "Read + Write" => 1,
        "Read" => 0,
        _ => 0,
    };

    let config = Configuration {
        base_path: settings.api_url.clone(),
        api_key: Some(trieve_client::apis::configuration::ApiKey {
            prefix: None,
            key: settings.api_key.clone(),
        }),
        ..Default::default()
    };

    let data = SetUserApiKeyParams {
        set_user_api_key_request: SetUserApiKeyRequest {
            name: name.clone(),
            role: role_num,
        },
    };

    let user = trieve_client::apis::user_api::set_user_api_key(&config, data)
        .await
        .map_err(|e| {
            eprintln!("Error generating API Key: {:?}", e);
            std::process::exit(1);
        })
        .unwrap()
        .entity
        .unwrap();

    match user {
        trieve_client::apis::user_api::SetUserApiKeySuccess::Status200(api_key) => {
            println!("\nAPI Key generated successfully!\n");
            println!("Name: {}", name);
            println!("API Key: {}", api_key.api_key);
        }
        trieve_client::apis::user_api::SetUserApiKeySuccess::UnknownValue(_) => {
            eprintln!("Error generating API Key.");
            std::process::exit(1);
        }
    }

    Ok(())
}
