use std::fmt;

use crate::Init;
use inquire::{Confirm, Text};
use serde::{Deserialize, Serialize};
use trieve_client::{
    apis::{
        auth_api::get_me,
        configuration::{ApiKey, Configuration},
    },
    models::Organization,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct TrieveConfiguration {
    pub api_key: String,
    pub organization_id: uuid::Uuid,
    pub api_url: String,
}

impl Default for TrieveConfiguration {
    fn default() -> Self {
        TrieveConfiguration {
            api_key: "".to_string(),
            organization_id: uuid::Uuid::nil(),
            api_url: "https://api.trieve.ai".to_string(),
        }
    }
}

struct OrgDTO(Organization);

impl fmt::Display for OrgDTO {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.0.name, self.0.id)
    }
}

async fn get_user(api_url: String, api_key: String) -> trieve_client::apis::auth_api::GetMeSuccess {
    let configuration = Configuration {
        base_path: api_url.clone(),
        api_key: Some(ApiKey {
            prefix: None,
            key: api_key.clone(),
        }),
        ..Default::default()
    };

    get_me(&configuration)
        .await
        .map_err(|e| {
            eprintln!("Error getting user: {:?}", e);
            std::process::exit(1);
        })
        .unwrap()
        .entity
        .unwrap()
}

async fn configure(api_url: String, mut api_key: Option<String>) -> TrieveConfiguration {
    if api_key.is_none() {
        let auth_url = format!(
            "{api_url}/api/auth?redirect_uri={api_url}/auth/cli%3Fhost={api_url}",
            api_url = api_url
        );
        println!(
            "\nPlease go to the following URL to get a Trieve API Key: {}\n",
            auth_url
        );

        api_key = Some(
            Text::new("API Key: ")
                .prompt()
                .map_err(|_| {
                    eprintln!("You must provide an API Key!");
                    std::process::exit(1);
                })
                .unwrap(),
        );
    }

    let result = get_user(api_url.clone(), api_key.clone().unwrap()).await;

    match result {
        trieve_client::apis::auth_api::GetMeSuccess::Status200(user) => {
            println!("\nWelcome, {}!", user.name.unwrap().unwrap());
            let orgs = user
                .orgs
                .iter()
                .map(|org| OrgDTO(org.clone()))
                .collect::<Vec<OrgDTO>>();

            let selected_organization =
                inquire::Select::new("Select an organization to use:", orgs)
                    .prompt()
                    .unwrap();

            TrieveConfiguration {
                api_key: api_key.unwrap(),
                organization_id: selected_organization.0.id,
                api_url: api_url.clone(),
            }
        }
        _ => {
            eprintln!("Error authenticating: {:?}", result);
            std::process::exit(1);
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ApiKeyResponse {
    api_key: String,
}

pub async fn init(init: Init, settings: TrieveConfiguration) {
    let api_key = init.api_key;
    let mut api_url = init.api_url;

    if settings.api_key.is_empty() && settings.organization_id.is_nil() {
        println!("Welcome to the Trieve CLI! Let's get started by configuring your API Key and Organization ID.");
    } else {
        println!("Welcome back to the Trieve CLI! Let's update your configuration.");
    }

    if api_url.is_none() {
        let use_prod = Confirm::new(
            "Would you like to use the production Trieve server (https://api.trieve.ai)?",
        )
        .with_default(true)
        .prompt();

        if use_prod.unwrap() {
            api_url = Some("https://api.trieve.ai".to_string());
        } else {
            api_url = Some(Text::new("Trieve Server URL: ").prompt().unwrap());
        }
    }

    let config = configure(api_url.unwrap().clone(), api_key).await;

    confy::store("trieve", None, config)
        .map_err(|e| {
            eprintln!("Error saving configuration: {:?}", e);
            std::process::exit(1);
        })
        .unwrap();
}

