use inquire::Confirm;
use serde::{Deserialize, Serialize};
use tabled::{builder::Builder, settings::Style};

use crate::{AddSeedData, CreateDataset, DeleteDataset};

use super::configure::TrieveConfiguration;

#[derive(Serialize, Deserialize, Debug)]
struct DatasetAndUsageDTO {
    dataset: DatasetDTO,
    dataset_usage: DatasetUsage,
}

impl std::fmt::Display for DatasetAndUsageDTO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.dataset.id, self.dataset.name,)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DatasetDTO {
    id: String,
    name: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    organization_id: String,
    client_configuration: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dataset {
    id: String,
    name: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    organization_id: String,
    client_configuration: serde_json::Value,
    server_configuration: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct DatasetUsage {
    chunk_count: u32,
    id: String,
    dataset_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CreateChunkData {
    chunk_html: String,
    link: String,
    tag_set: Vec<String>,
    tracking_id: String,
    metadata: serde_json::Value,
}

fn get_datasets_from_org(
    settings: TrieveConfiguration,
) -> Result<Vec<DatasetAndUsageDTO>, ureq::Error> {
    let response = ureq::get(&format!(
        "{}/api/dataset/organization/{}",
        settings.api_url, settings.organization_id
    ))
    .set("Authorization", &settings.api_key)
    .set("TR-Organization", &settings.organization_id)
    .call()?;

    if response.status() != 200 {
        eprintln!(
            "Error listing datasets: {}",
            response.into_string().unwrap()
        );
        std::process::exit(1);
    }

    let datasets: Vec<DatasetAndUsageDTO> =
        serde_json::from_str(response.into_string().unwrap().as_str()).unwrap(); //TODO: Handle errors

    Ok(datasets)
}

pub fn list_datasets(settings: TrieveConfiguration) -> Result<(), ureq::Error> {
    if settings.organization_id.is_empty() || settings.api_key.is_empty() {
        eprintln!("Please configure the Trieve CLI with your credentials. Run `trieve configure` to get started.");
        std::process::exit(1);
    }

    let datasets = get_datasets_from_org(settings.clone())?;

    let mut builder = Builder::default();

    builder.push_record(["ID", "Name", "Created At", "Updated At", "Chunk Count"]);

    for dataset in datasets {
        builder.push_record([
            dataset.dataset.id,
            dataset.dataset.name,
            dataset.dataset.created_at.date().to_string(),
            dataset.dataset.updated_at.date().to_string(),
            dataset.dataset_usage.chunk_count.to_string(),
        ]);
    }

    let table = builder.build().with(Style::rounded()).to_string();
    println!("Datasets for organization: {}", settings.organization_id);
    println!("{}", table);
    Ok(())
}

pub fn create_dataset(
    settings: TrieveConfiguration,
    create: CreateDataset,
) -> Result<Dataset, ureq::Error> {
    if settings.organization_id.is_empty() || settings.api_key.is_empty() {
        eprintln!("Please configure the Trieve CLI with your credentials. Run `trieve configure` to get started.");
        std::process::exit(1);
    }

    let mut name = create.name.clone();
    if create.name.is_none() {
        name = Some(inquire::Text::new("Dataset Name: ").prompt().unwrap());
    }

    let response = ureq::post(&format!("{}/api/dataset", settings.api_url))
        .set("Authorization", &settings.api_key)
        .set("TR-Organization", &settings.organization_id)
        .send_json(serde_json::json!({
            "dataset_name": name.unwrap(),
            "organization_id": settings.organization_id,
            "client_configuration": {},
            "server_configuration": {
                "LLM_BASE_URL": "",
                "LLM_DEFAULT_MODEL": "",
                "EMBEDDING_BASE_URL": "https://embedding.trieve.ai",
                "RAG_PROMPT": "",
                "EMBEDDING_SIZE": 768,
                "N_RETRIEVALS_TO_INCLUDE": 8,
                "DUPLICATE_DISTANCE_THRESHOLD": 1.1,
                "DOCUMENT_UPLOAD_FEATURE": true,
                "DOCUMENT_DOWNLOAD_FEATURE": true,
                "COLLISIONS_ENABLED": false,
                "FULLTEXT_ENABLED": true,
            },
        }))?;

    if response.status() != 200 {
        eprintln!(
            "Error creating dataset: {}",
            response.into_string().unwrap()
        );
        std::process::exit(1);
    }

    let dataset: Dataset = serde_json::from_str(response.into_string().unwrap().as_str()).unwrap(); //TODO: Handle errors

    println!("Dataset created successfully!");
    println!("");
    println!("ID: {}", dataset.id);
    println!("Name: {}", dataset.name);
    println!("Organization ID: {}", dataset.organization_id);

    Ok(dataset)
}

pub fn delete_dataset(
    settings: TrieveConfiguration,
    delete: DeleteDataset,
) -> Result<(), ureq::Error> {
    if settings.organization_id.is_empty() || settings.api_key.is_empty() {
        eprintln!("Please configure the Trieve CLI with your credentials. Run `trieve configure` to get started.");
        std::process::exit(1);
    }

    let mut dataset_id = delete.dataset_id.clone();

    if dataset_id.is_none() {
        let datasets = get_datasets_from_org(settings.clone())?;

        let selected_dataset = inquire::Select::new("Select a dataset to delete:", datasets)
            .prompt()
            .unwrap();

        let ans = Confirm::new("Are you sure you want to delete this dataset?")
            .with_default(false)
            .prompt();

        if !ans.unwrap() {
            println!("Dataset deletion cancelled.");
            std::process::exit(0);
        }

        dataset_id = Some(selected_dataset.dataset.id);
    }

    let response = ureq::delete(&format!(
        "{}/api/dataset/{}",
        settings.api_url,
        dataset_id.clone().unwrap()
    ))
    .set("Authorization", &settings.api_key)
    .set("TR-Dataset", &dataset_id.clone().unwrap())
    .call()?;

    if response.status() != 200 {
        eprintln!(
            "Error deleting dataset: {}",
            response.into_string().unwrap()
        );
        std::process::exit(1);
    }

    println!("Dataset deleted successfully!");

    Ok(())
}

pub async fn add_seed_data(
    settings: TrieveConfiguration,
    seed_data: AddSeedData,
) -> Result<(), ureq::Error> {
    if settings.organization_id.is_empty() || settings.api_key.is_empty() {
        eprintln!("Please configure the Trieve CLI with your credentials. Run `trieve configure` to get started.");
        std::process::exit(1);
    }

    let mut dataset_id = seed_data.dataset_id.clone();

    if dataset_id.is_none() {
        let datasets = get_datasets_from_org(settings.clone())?;

        let ans = Confirm::new("Would you like to create a new dataset?")
            .with_default(true)
            .with_help_message("If you select No, you will be prompted to select an existing dataset to add seed data to.")
            .prompt();

        if ans.unwrap() {
            let create = CreateDataset { name: None };

            let dataset = create_dataset(settings.clone(), create)?;
            dataset_id = Some(dataset.id);
        } else {
            let selected_dataset =
                inquire::Select::new("Select a dataset to add seed data to:", datasets)
                    .prompt()
                    .unwrap();
            dataset_id = Some(selected_dataset.dataset.id);
        }
    }

    let response = ureq::get("https://gist.githubusercontent.com/densumesh/127bd58e026ccadaea58dc1aa3ad9648/raw/1dcf2fe14954047064ef5cfbec43bf74d54365d8/yc-company-data.csv").call()?;
    let mut rdr = csv::Reader::from_reader(response.into_reader());

    let chunk_data: Vec<CreateChunkData> = rdr
        .records()
        .map(|record| {
            let record = record.expect("Error reading CSV record");
            let chunk_data = CreateChunkData {
                chunk_html: record[0].to_string().replace(";", ","),
                link: record[1].to_string().replace(";", ","),
                tag_set: record[2].split("|").map(|s| s.to_string()).collect(),
                tracking_id: record[3].to_string(),
                metadata: record[4].to_string().replace(";", ",").parse().unwrap(),
            };
            chunk_data
        })
        .collect();

    println!(
        "Adding seed data to dataset: {}",
        dataset_id.clone().unwrap()
    );

    let mut handles = vec![];

    for chunk in chunk_data.chunks(30) {
        let settings = settings.clone();
        let dataset_id = dataset_id.clone();
        let chunk = chunk.to_vec();
        let handle = tokio::spawn(async move {
            let response = ureq::post(&format!("{}/api/chunk", settings.api_url))
                .set("Authorization", &settings.api_key)
                .set("TR-Dataset", &dataset_id.clone().unwrap())
                .send_json(chunk);

            if let Err(e) = response {
                eprintln!("Request failed: {}", e);
            } else if let Ok(resp) = response {
                if resp.status() != 200 {
                    eprintln!("Error adding seed data: {}", resp.into_string().unwrap());
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("Seed data added successfully! Access your dataset at: https://search.trieve.ai/?dataset={}", dataset_id.clone().unwrap());
    Ok(())
}
