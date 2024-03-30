use serde::{Deserialize, Serialize};
use tabled::{builder::Builder, settings::Style};

use crate::{CreateDataset, DeleteDataset};

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
struct Dataset {
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
) -> Result<(), ureq::Error> {
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
            "server_configuration": {},
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

    Ok(())
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

        dataset_id = Some(selected_dataset.dataset.id);
    }

    let response = ureq::delete(&format!("{}/api/dataset", settings.api_url))
        .set("Authorization", &settings.api_key)
        .set("TR-Dataset", &dataset_id.clone().unwrap())
        .send_json(serde_json::json!({
            "dataset_id": dataset_id.unwrap(),
        }))?;

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
