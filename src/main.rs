use clap::{Args, Parser, Subcommand};
use commands::configure::TrieveConfiguration;

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
}

#[derive(Subcommand)]
enum Commands {
    /// Configures the Trieve CLI with your API key
    Init(Init),
    /// Commands for interacting with datasets in the Trieve service
    #[command(subcommand)]
    Dataset(DatasetCommands),
    //TODO: add command to generate api key
    #[command(subcommand)]
    Generate(Generate),
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

#[derive(Subcommand)]
enum Generate {
    /// Generate an API key for the Trieve service
    ApiKey(ApiKey),
}

#[derive(Args)]
struct Init {
    /// API Key from the Trieve dashboard (https://dashboard.trieve.ai)
    #[arg(short, long, env = "TRIEVE_API_KEY")]
    api_key: Option<String>,
    /// The URL of the Trieve server if you are using a self-hosted version of Trieve
    #[arg(long, required = false)]
    api_url: Option<String>,
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
struct ApiKey;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let settings: TrieveConfiguration = confy::load("trieve", None)
        .map_err(|e| {
            eprintln!("Error loading configuration: {:?}", e);
            std::process::exit(1);
        })
        .unwrap();

    match args.command {
        Some(Commands::Init(init)) => {
            commands::configure::init(init, settings).await;
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
        Some(Commands::Generate(generate)) => match generate {
            Generate::ApiKey(_) => {
                commands::generate::generate_api_key(settings)
                    .await
                    .map_err(|e| {
                        eprintln!("Error generating API key: {:?}", e);
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
