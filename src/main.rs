use crate::commands::configure::TrieveConfiguration;
use clap::{Args, Parser, Subcommand};
use commands::configure::TrieveProfile;
use std::env;

mod commands;

#[derive(Parser)]
#[command(author, version)]
#[command(
    name = "trieve",
    about = "Trieve CLI - CLI for Trieve Search Product",
    long_about = "Trieve CLI is a CLI for the Trieve Search Product. 
    
    It allows you to interact with the Trieve Search Product from the command line by adding data."
)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// The name of the profile to use
    #[arg(short, long, env = "TRIEVE_PROFILE")]
    profile: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configures the Trieve CLI with your API key
    Login(Login),
    /// Commands for interacting with datasets in the Trieve service
    #[command(subcommand)]
    Dataset(DatasetCommands),
    #[command(subcommand, about = "Commands for managing API Keys")]
    ApiKey(ApiKeyCommands),
    /// Command to manage profiles
    #[command(subcommand)]
    Profile(Profile),
    /// Command to interact with organizations
    #[command(subcommand)]
    Organization(Organization),
}

#[derive(Subcommand)]
enum Profile {
    /// Switch to a different profile
    Switch(SwitchProfile),
    /// Delete a profile
    Delete(DeleteProfile),
    /// List all profiles
    List(ListProfile),
}

#[derive(Subcommand)]
enum Organization {
    /// Switch to a different organization
    Switch(SwitchOrganization),
    /// Create an organization
    Create(CreateOrganization),
    /// Delete an organization
    Delete(DeleteOrganization),
}

#[derive(Subcommand)]
enum ApiKeyCommands {
    /// Generate a new API Key
    Generate(ApiKeyData),
    //TODO: List API Keys, Delete API Key
}

#[derive(Subcommand)]
enum DatasetCommands {
    /// Create a dataset in the Trieve service
    Create(CreateDataset),
    /// List all datasets in the Trieve service
    List(ListDatasets),
    /// Delete a dataset in the Trieve service
    Delete(DeleteDataset),
    /// Add seed data to a dataset in the Trieve service
    Example(AddSeedData),
}

#[derive(Args)]
struct Login {
    /// API Key from the Trieve dashboard (https://dashboard.trieve.ai)
    #[arg(short, long, env = "TRIEVE_API_KEY")]
    api_key: Option<String>,
    /// The URL of the Trieve server if you are using a self-hosted version of Trieve
    #[arg(long, required = false)]
    api_url: Option<String>,
    /// Name the profile you are configuring
    #[arg(long, required = false)]
    profile_name: Option<String>,
}

#[derive(Args)]
struct CreateDataset {
    /// The name of the dataset
    #[arg(short, long)]
    name: Option<String>,
}

#[derive(Args)]
struct ListDatasets;

#[derive(Args)]
struct DeleteDataset {
    /// The ID of the dataset to delete
    dataset_id: Option<String>,
}

#[derive(Args)]
struct AddSeedData {
    /// The ID of the dataset to add seed data to
    dataset_id: Option<String>,
}

#[derive(Args)]
struct ApiKeyData {
    /// The name of the API Key
    #[arg(short, long)]
    name: Option<String>,
    /// The role of the API Key
    #[arg(short, long)]
    role: Option<String>,
}

#[derive(Args)]
struct SwitchProfile {
    /// The name of the profile to switch to
    profile_name: Option<String>,
}

#[derive(Args)]
struct DeleteProfile {
    /// The name of the profile to delete
    profile_name: Option<String>,
}

#[derive(Args)]
struct ListProfile;

#[derive(Args)]
struct SwitchOrganization {
    /// The ID of the organization to switch to
    organization_id: Option<String>,
}

#[derive(Args)]
struct CreateOrganization {
    /// The name of the organization to create
    name: Option<String>,
}

#[derive(Args)]
struct DeleteOrganization {
    /// The ID of the organization to delete
    organization_id: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let no_profile =
        env::var("TRIEVE_NO_PROFILE").unwrap_or_else(|_| String::new()) == "true";

    let profiles: TrieveProfile = confy::load("trieve", "profiles")
        .map_err(|e| {
            eprintln!("Error loading configuration: {:?}", e);
        })
        .unwrap_or_default();

    let settings = if no_profile {
        TrieveConfiguration::from_env().unwrap_or_else(|e| {
            eprintln!(
                "Error creating configuration from environment variables: {:?}",
                e
            );
            std::process::exit(1);
        })
    } else if args.profile.is_some() {
        let profile_name = args.profile.unwrap();
        let profile = profiles
            .inner
            .iter()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| {
                eprintln!("Profile '{}' not found.", profile_name);
                std::process::exit(1);
            })
            .unwrap();

        profile.settings.clone()
    } else {
        profiles
            .inner
            .iter()
            .find(|p| p.selected)
            .cloned()
            .unwrap_or_default()
            .settings
    };

    match args.command {
        Some(Commands::Login(login)) => {
            commands::configure::login(login, settings).await;
        }
        Some(Commands::Dataset(dataset)) => match dataset {
            DatasetCommands::List(_) => commands::dataset::list_datasets(settings)
                .await
                .map_err(|e| {
                    eprintln!("Error listing datasets: {:?}", e);
                    std::process::exit(1);
                })
                .unwrap(),
            DatasetCommands::Create(create) => {
                commands::dataset::create_trieve_dataset(settings, create)
                    .await
                    .map_err(|e| {
                        eprintln!("Error creating dataset: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
            DatasetCommands::Delete(delete) => {
                commands::dataset::delete_trieve_dataset(settings, delete)
                    .await
                    .map_err(|e| {
                        eprintln!("Error deleting dataset: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
            DatasetCommands::Example(seed_data) => {
                commands::dataset::add_seed_data(settings, seed_data)
                    .await
                    .map_err(|e| {
                        eprintln!("Error adding seed data: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
        },
        Some(Commands::ApiKey(api_key)) => match api_key {
            ApiKeyCommands::Generate(api_key_data) => {
                commands::api_key::generate_api_key(settings, api_key_data)
                    .await
                    .map_err(|e| {
                        eprintln!("Error generating API Key: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
        },
        Some(Commands::Profile(profile)) => match profile {
            Profile::Switch(switch) => {
                commands::profile::switch_profile(switch, profiles.to_vec())
                    .map_err(|e| {
                        eprintln!("Error switching profile: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
            Profile::Delete(delete) => {
                commands::profile::delete_profile(delete, profiles.to_vec())
                    .map_err(|e| {
                        eprintln!("Error deleting profile: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
            Profile::List(_) => {
                commands::profile::list_profiles(profiles.to_vec())
                    .map_err(|e| {
                        eprintln!("Error listing profiles: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
        },
        Some(Commands::Organization(organization)) => match organization {
            Organization::Switch(switch) => {
                commands::organization::switch_organization(switch, profiles.to_vec(), settings)
                    .await
                    .map_err(|e| {
                        eprintln!("Error switching organization: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
            Organization::Create(create) => {
                commands::organization::create_organization(create, settings)
                    .await
                    .map_err(|e| {
                        eprintln!("Error creating organization: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
            Organization::Delete(delete) => {
                commands::organization::delete_organization(delete, settings)
                    .await
                    .map_err(|e| {
                        eprintln!("Error deleting organization: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
        },
        _ => {
            println!("Command not implemented yet");
        }
    }
}
