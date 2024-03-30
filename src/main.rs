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
    Configure(Configure),
    /// Commands for interacting with datasets in the Trieve service
    #[command(subcommand)]
    Dataset(DatasetCommands),
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
struct Configure {
    /// API Key from the Trieve dashboard (https://dashboard.trieve.ai)
    #[arg(short, long, env = "TRIEVE_API_KEY")]
    api_key: Option<String>,
    /// Organization ID from the Trieve dashboard (https://dashboard.trieve.ai)
    #[arg(short, long, env = "TRIEVE_ORGANIZATION_ID")]
    organization_id: Option<String>,
    /// The URL of the Trieve server if you are using a self-hosted version of Trieve
    #[arg(long, default_value = "https://api.trieve.ai", required = false)]
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
    /// The string to reverse
    dataset_id: Option<String>,
}

#[derive(Args)]
struct AddSeedData {
    /// The string to reverse
    string: Option<String>,
}

fn main() {
    let args = Cli::parse();

    let settings: TrieveConfiguration = confy::load("trieve", None)
        .map_err(|e| {
            eprintln!("Error loading configuration: {:?}", e);
            std::process::exit(1);
        })
        .unwrap();

    match args.command {
        Some(Commands::Configure(configure)) => {
            commands::configure::parse_configuration(configure);
        }
        Some(Commands::Dataset(dataset)) => match dataset {
            DatasetCommands::List(_) => commands::dataset::list_datasets(settings)
                .map_err(|e| {
                    eprintln!("Error listing datasets: {:?}", e);
                    std::process::exit(1);
                })
                .unwrap(),
            DatasetCommands::Create(create) => {
                commands::dataset::create_dataset(settings, create)
                    .map_err(|e| {
                        eprintln!("Error creating dataset: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
            DatasetCommands::Delete(delete) => {
                commands::dataset::delete_dataset(settings, delete)
                    .map_err(|e| {
                        eprintln!("Error deleting dataset: {:?}", e);
                        std::process::exit(1);
                    })
                    .unwrap();
            }
            _ => {
                println!("Command not implemented yet");
            }
        },
        _ => {
            println!("Command not implemented yet");
        }
    }
}
