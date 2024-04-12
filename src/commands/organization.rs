use crate::{
    commands::configure::{get_user, OrgDTO},
    SwitchOrganization,
};

use super::configure::{TrieveConfiguration, TrieveProfileInner};

pub async fn switch_organization(
    organization_data: SwitchOrganization,
    profiles: Vec<TrieveProfileInner>,
    settings: TrieveConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization_id = if organization_data.organization_id.is_none() {
        let result = get_user(settings.api_url.clone(), settings.api_key.clone()).await;

        match result {
            trieve_client::apis::auth_api::GetMeSuccess::Status200(user) => {
                let orgs = user
                    .orgs
                    .iter()
                    .map(|org| OrgDTO(org.clone()))
                    .collect::<Vec<OrgDTO>>();

                let selected_organization =
                    inquire::Select::new("Select an organization to use:", orgs)
                        .prompt()
                        .unwrap();

                selected_organization.0.id
            }
            _ => {
                eprintln!("Error authenticating: {:?}", result);
                std::process::exit(1);
            }
        }
    } else {
        organization_data
            .organization_id
            .unwrap()
            .parse()
            .map_err(|e| {
                eprintln!("Invalid organization ID: {:?}", e);
                std::process::exit(1);
            })
            .unwrap()
    };

    let new_config = TrieveConfiguration {
        api_key: settings.api_key.clone(),
        organization_id,
        api_url: settings.api_url.clone(),
    };

    let profiles = profiles
        .iter()
        .map(|p| {
            if p.settings == settings {
                TrieveProfileInner {
                    name: p.name.clone(),
                    selected: true,
                    settings: new_config.clone(),
                }
            } else {
                TrieveProfileInner {
                    name: p.name.clone(),
                    selected: false,
                    settings: p.settings.clone(),
                }
            }
        })
        .collect::<Vec<TrieveProfileInner>>();

    confy::store("trieve", "profiles", profiles)
        .map_err(|e| {
            eprintln!("Error saving configuration: {:?}", e);
            std::process::exit(1);
        })
        .unwrap();

    println!("Switched to organization '{}'.", organization_id);

    Ok(())
}
