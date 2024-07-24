use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::{commands::login_server::server, Login};
use inquire::{Confirm, Text};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use trieve_client::{
    apis::{
        auth_api::get_me,
        configuration::{ApiKey, Configuration},
    },
    models::{Organization, SlimUser},
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct TrieveConfiguration {
    pub api_key: String,
    pub organization_id: uuid::Uuid,
    pub api_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrieveProfileInner {
    pub name: String,
    pub settings: TrieveConfiguration,
    pub selected: bool,
}

impl Default for TrieveProfileInner {
    fn default() -> Self {
        TrieveProfileInner {
            name: "default".to_string(),
            settings: TrieveConfiguration::default(),
            selected: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrieveProfile {
    pub inner: Vec<TrieveProfileInner>,
}

impl Default for TrieveProfile {
    fn default() -> Self {
        TrieveProfile {
            inner: vec![TrieveProfileInner::default()],
        }
    }
}

impl Deref for TrieveProfile {
    type Target = Vec<TrieveProfileInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TrieveProfile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrgDTO(pub Organization);

impl fmt::Display for OrgDTO {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.0.name, self.0.id)
    }
}

pub async fn get_user(api_url: String, api_key: String) -> SlimUser {
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
}

async fn configure(api_url: String, mut api_key: Option<String>) -> TrieveConfiguration {
    if api_key.is_none() {
        let (tx, mut rx) = mpsc::channel::<String>(100);

        let server = tokio::spawn(async move {
            server(tx.clone()).await.map_err(|e| {
                eprintln!("Error starting server: {:?}", e);
                std::process::exit(1);
            })
        });

        let auth_url = format!(
            "{api_url}/api/auth?redirect_uri={api_url}/auth/cli%3Fhost={api_url}",
            api_url = api_url
        );

        let _ = Text::new("Press Enter to authenticate in browser: ")
            .prompt()
            .unwrap();

        if open::that(auth_url.clone()).is_err() {
            eprintln!("Error opening browser. Please visit the URL manually.");
            println!(
                "\nPlease go to the following URL to get a Trieve API Key: {}",
                auth_url
            );
        }

        api_key = Some(rx.recv().await.unwrap());

        server.abort();
    }

    let user = get_user(api_url.clone(), api_key.clone().unwrap()).await;

    println!("\nWelcome, {}!", user.name.unwrap().unwrap());
    let orgs = user
        .orgs
        .iter()
        .map(|org| OrgDTO(org.clone()))
        .collect::<Vec<OrgDTO>>();
    let selected_organization = inquire::Select::new("Select an organization to use:", orgs)
        .prompt()
        .unwrap();

    TrieveConfiguration {
        api_key: api_key.unwrap(),
        organization_id: selected_organization.0.id,
        api_url: api_url.clone(),
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ApiKeyResponse {
    api_key: String,
}

pub async fn login(init: Login, settings: TrieveConfiguration) {
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
            api_url = Some(
                Text::new("Trieve Server URL: ")
                    .with_default("http://localhost:8090")
                    .prompt()
                    .unwrap(),
            );
        }
    }

    let config = configure(api_url.unwrap().clone(), api_key).await;

    let profile_name = if init.profile_name.is_none() {
        let profile_name = Text::new("Enter a name for this profile:")
            .with_default("default")
            .prompt()
            .unwrap();
        println!(
            "Configuration complete! Your profile has been saved as '{}'.",
            profile_name
        );
        profile_name
    } else {
        init.profile_name.unwrap()
    };

    let mut profiles: TrieveProfile = confy::load("trieve", "profiles")
        .map_err(|e| {
            eprintln!("Error loading configuration: {:?}", e);
        })
        .unwrap_or_default();

    if profiles
        .iter()
        .any(|p| p.name == profile_name && p.settings.organization_id != uuid::Uuid::nil())
    {
        let overwrite = Confirm::new("Profile already exists. Overwrite?")
            .with_default(false)
            .prompt();

        if !overwrite.unwrap() {
            std::process::exit(0);
        }

        profiles.retain(|p| p.name != profile_name);
    }

    profiles.dedup_by_key(|p| p.name.clone());
    profiles.iter_mut().for_each(|p| p.selected = false);

    profiles.push(TrieveProfileInner {
        name: profile_name,
        settings: config,
        selected: true,
    });

    confy::store("trieve", "profiles", profiles)
        .map_err(|e| {
            eprintln!("Error saving configuration: {:?}", e);
            std::process::exit(1);
        })
        .unwrap();
}
