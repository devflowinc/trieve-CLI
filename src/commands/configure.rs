use crate::Configure;
use inquire::Text;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TrieveConfiguration {
    pub api_key: String,
    pub organization_id: String,
    pub api_url: String,
}

impl Default for TrieveConfiguration {
    fn default() -> Self {
        TrieveConfiguration {
            api_key: "".to_string(),
            organization_id: "".to_string(),
            api_url: "https://api.trieve.ai".to_string(),
        }
    }
}

pub fn parse_configuration(configuration: Configure) {
    let mut api_key = configuration.api_key;
    let mut organization_id = configuration.organization_id;
    if api_key.is_none() {
        println!("An API Key is required to use the Trieve CLI. You can find your API Key in the Trieve dashboard at https://dashboard.trieve.ai.");
        api_key = Some(Text::new("API Key: ").prompt().unwrap());
    }
    if organization_id.is_none() {
        println!("An Organization ID is required to use the Trieve CLI. You can find your Organization ID in the Trieve dashboard at https://dashboard.trieve.ai.");
        organization_id = Some(Text::new("Organization ID: ").prompt().unwrap());
    }

    let settings = TrieveConfiguration {
        api_key: api_key.unwrap(),
        organization_id: organization_id.unwrap(),
        api_url: configuration.api_url.unwrap(),
    };

    confy::store("trieve", None, settings)
        .map_err(|e| {
            eprintln!("Error saving configuration: {:?}", e);
            std::process::exit(1);
        })
        .unwrap();
}
