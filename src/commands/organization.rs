use trieve_client::{
    apis::configuration::{ApiKey, Configuration},
    models::CreateOrganizationReqPayload,
};

use crate::{
    commands::configure::{get_user, OrgDTO, TrieveProfile},
    CreateOrganization, DeleteOrganization, SwitchOrganization,
};

use super::configure::{TrieveConfiguration, TrieveProfileInner};

pub async fn switch_organization(
    organization_data: SwitchOrganization,
    profiles: Vec<TrieveProfileInner>,
    settings: TrieveConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization_id = if organization_data.organization_id.is_none() {
        let user = get_user(settings.api_url.clone(), settings.api_key.clone()).await;

        let orgs = user
            .orgs
            .iter()
            .map(|org| OrgDTO(org.clone()))
            .collect::<Vec<OrgDTO>>();

        let selected_organization =
            inquire::Select::new("Select an organization to use:", orgs.clone())
                .with_starting_cursor(
                    orgs.iter()
                        .position(|o| {
                            o.0.id
                                == profiles
                                    .iter()
                                    .find(|p| p.selected)
                                    .map(|p| p.settings.organization_id)
                                    .unwrap_or_default()
                        })
                        .unwrap_or(0),
                )
                .prompt()
                .unwrap();

        selected_organization.0.id
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

    confy::store("trieve", "profiles", TrieveProfile { inner: profiles })
        .map_err(|e| {
            eprintln!("Error saving configuration: {:?}", e);
            std::process::exit(1);
        })
        .unwrap();

    println!("Switched to organization '{}'.", organization_id);

    Ok(())
}

pub async fn create_organization(
    organization_data: CreateOrganization,
    settings: TrieveConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = if organization_data.name.is_none() {
        inquire::Text::new("Enter a name for this organization:")
            .prompt()
            .unwrap()
    } else {
        organization_data.name.unwrap()
    };
    let configuration = Configuration {
        base_path: settings.api_url.clone(),
        api_key: Some(ApiKey {
            prefix: None,
            key: settings.api_key.clone(),
        }),
        ..Default::default()
    };

    let org = trieve_client::apis::organization_api::create_organization(
        &configuration,
        CreateOrganizationReqPayload { name },
    )
    .await
    .map_err(|e| {
        eprintln!("Error getting user: {:?}", e);
        std::process::exit(1);
    })
    .unwrap();

    println!("Organization '{}' created.", org.id);
    Ok(())
}

pub async fn delete_organization(
    data: DeleteOrganization,
    settings: TrieveConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization_id = if data.organization_id.is_none() {
        let user = get_user(settings.api_url.clone(), settings.api_key.clone()).await;

        let orgs = user
            .orgs
            .iter()
            .map(|org| OrgDTO(org.clone()))
            .collect::<Vec<OrgDTO>>();

        let selected_organization =
            inquire::Select::new("Select an organization to delete:", orgs.clone())
                .prompt()
                .unwrap();

        selected_organization.0.id.to_string()
    } else {
        data.organization_id.unwrap()
    };

    let configuration = Configuration {
        base_path: settings.api_url.clone(),
        api_key: Some(ApiKey {
            prefix: None,
            key: settings.api_key.clone(),
        }),
        ..Default::default()
    };

    trieve_client::apis::organization_api::delete_organization(
        &configuration,
        &organization_id.clone(),
        &organization_id.clone(),
    )
    .await
    .map_err(|e| {
        eprintln!("Error getting organization: {:?}", e);
        std::process::exit(1);
    })
    .unwrap();

    Ok(())
}
