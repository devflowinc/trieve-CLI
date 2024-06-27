use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use inquire::Confirm;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tabled::{builder::Builder, settings::Style};
use trieve_client::{
    apis::{
        chunk_api::{create_chunk, CreateChunkParams},
        chunk_group_api::create_chunk_group,
        configuration::{ApiKey, Configuration},
        dataset_api::{
            create_dataset, delete_dataset, get_datasets_from_organization, CreateDatasetParams,
            DeleteDatasetParams, GetDatasetsFromOrganizationParams,
        },
    },
    models::{ChunkData, CreateChunkGroupData, CreateDatasetRequest, Dataset, DatasetAndUsage},
};

use crate::{AddSeedData, CreateDataset, DeleteDataset};

use super::configure::TrieveConfiguration;
use std::{collections::HashSet, fmt};

struct DatasetAndUsageDTO(DatasetAndUsage);

impl fmt::Display for DatasetAndUsageDTO {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.0.dataset.name, self.0.dataset.id)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DatasetUsage {
    chunk_count: u32,
    id: String,
    dataset_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DefaultError {
    message: String,
}

async fn get_datasets_from_org(
    settings: TrieveConfiguration,
) -> Result<Vec<DatasetAndUsage>, DefaultError> {
    let config = Configuration {
        base_path: settings.api_url,
        api_key: Some(ApiKey {
            prefix: None,
            key: settings.api_key,
        }),
        ..Default::default()
    };

    let data = GetDatasetsFromOrganizationParams {
        tr_organization: settings.organization_id.to_string().clone(),
        organization_id: settings.organization_id.to_string(),
    };
    let result = get_datasets_from_organization(&config, data)
        .await
        .map_err(|e| DefaultError {
            message: e.to_string(),
        })?
        .entity
        .unwrap();

    match result {
        trieve_client::apis::dataset_api::GetDatasetsFromOrganizationSuccess::Status200(
            datasets,
        ) => Ok(datasets),
        trieve_client::apis::dataset_api::GetDatasetsFromOrganizationSuccess::UnknownValue(val) => {
            Err(DefaultError {
                message: format!("Error getting datasets: {}", val),
            })
        }
    }
}

pub async fn list_datasets(settings: TrieveConfiguration) -> Result<(), DefaultError> {
    if settings.organization_id.to_string().is_empty() || settings.api_key.is_empty() {
        eprintln!("Please login to the Trieve CLI with your credentials. Run `trieve login` to get started.");
        std::process::exit(1);
    }

    let datasets = get_datasets_from_org(settings.clone()).await.map_err(|e| {
        eprintln!("Error listing datasets: {}", e.message);
        std::process::exit(1);
    })?;

    let mut builder = Builder::default();

    builder.push_record(["ID", "Name", "Created At", "Updated At", "Chunk Count"]);

    for dataset in datasets {
        builder.push_record([
            dataset.dataset.id.to_string(),
            dataset.dataset.name,
            dataset
                .dataset
                .created_at
                .parse::<NaiveDateTime>()
                .unwrap()
                .date()
                .to_string(),
            dataset
                .dataset
                .updated_at
                .parse::<NaiveDateTime>()
                .unwrap()
                .date()
                .to_string(),
            dataset.dataset_usage.chunk_count.to_string(),
        ]);
    }

    let table = builder.build().with(Style::rounded()).to_string();
    println!("Datasets for organization: {}", settings.organization_id);
    println!("{}", table);
    Ok(())
}

pub async fn create_trieve_dataset(
    settings: TrieveConfiguration,
    create: CreateDataset,
) -> Result<Dataset, DefaultError> {
    if settings.organization_id.to_string().is_empty() || settings.api_key.is_empty() {
        eprintln!("Please login to the Trieve CLI with your credentials. Run `trieve login` to get started.");
        std::process::exit(1);
    }

    let mut name = create.name.clone();
    if create.name.is_none() {
        name = Some(inquire::Text::new("Dataset Name: ").prompt().unwrap());
    }

    let config = Configuration {
        base_path: settings.api_url,
        api_key: Some(ApiKey {
            prefix: None,
            key: settings.api_key,
        }),
        ..Default::default()
    };

    let data = CreateDatasetParams {
        tr_organization: settings.organization_id.to_string().clone(),
        create_dataset_request: CreateDatasetRequest {
            organization_id: settings.organization_id,
            dataset_name: name.unwrap(),
            client_configuration: Some(serde_json::json!({})),
            server_configuration: Some(serde_json::json!({
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
            })),
            tracking_id: None,
        },
    };

    let result = create_dataset(&config, data)
        .await
        .map_err(|e| DefaultError {
            message: e.to_string(),
        })?
        .entity
        .unwrap();

    let dataset = match result {
        trieve_client::apis::dataset_api::CreateDatasetSuccess::Status200(dataset) => Ok(dataset),
        trieve_client::apis::dataset_api::CreateDatasetSuccess::UnknownValue(val) => {
            Err(DefaultError {
                message: format!("Error creating dataset: {}", val),
            })
        }
    }?;

    println!("Dataset created successfully!");
    println!();
    println!("ID: {}", dataset.id);
    println!("Name: {}", dataset.name);
    println!("Organization ID: {}", dataset.organization_id);

    Ok(dataset)
}

pub async fn delete_trieve_dataset(
    settings: TrieveConfiguration,
    delete: DeleteDataset,
) -> Result<(), DefaultError> {
    if settings.organization_id.to_string().is_empty() || settings.api_key.is_empty() {
        eprintln!("Please login to the Trieve CLI with your credentials. Run `trieve login` to get started.");
        std::process::exit(1);
    }

    let mut dataset_id = delete.dataset_id.clone();

    if dataset_id.is_none() {
        let datasets = get_datasets_from_org(settings.clone())
            .await
            .map_err(|e| {
                eprintln!("Error listing datasets: {}", e.message);
                std::process::exit(1);
            })?
            .iter()
            .map(|d| DatasetAndUsageDTO(d.clone()))
            .collect::<Vec<_>>();

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

        dataset_id = Some(selected_dataset.0.dataset.id.to_string());
    }

    let config = Configuration {
        base_path: settings.api_url,
        api_key: Some(ApiKey {
            prefix: None,
            key: settings.api_key,
        }),
        ..Default::default()
    };

    let data = DeleteDatasetParams {
        tr_dataset: dataset_id.clone().unwrap(),
        dataset_id: dataset_id.clone().unwrap(),
    };

    let result = delete_dataset(&config, data)
        .await
        .map_err(|e| DefaultError {
            message: e.to_string(),
        })?
        .entity;

    match result {
        Some(trieve_client::apis::dataset_api::DeleteDatasetSuccess::Status204()) => (),
        Some(trieve_client::apis::dataset_api::DeleteDatasetSuccess::UnknownValue(val)) => {
            return Err(DefaultError {
                message: format!("Error deleting dataset: {}", val),
            });
        }
        None => (),
    };

    println!("Dataset deleted successfully!");

    Ok(())
}

async fn add_yc_companies_seed_data(
    settings: TrieveConfiguration,
    dataset_id: Option<String>,
) -> Result<(), DefaultError> {
    let response = ureq::get("https://gist.githubusercontent.com/densumesh/127bd58e026ccadaea58dc1aa3ad9648/raw/1dcf2fe14954047064ef5cfbec43bf74d54365d8/yc-company-data.csv").call().map_err(
        |e| DefaultError {
            message: e.to_string(),
        }
    )?;

    let mut rdr = csv::Reader::from_reader(response.into_reader());

    let chunk_data: Vec<ChunkData> = rdr
        .records()
        .map(|record| {
            let record = record.expect("Error reading CSV record");
            let chunk_data = ChunkData {
                chunk_html: Some(Some(record[0].to_string().replace(';', ","))),
                link: Some(Some(record[1].to_string().replace(';', ","))),
                tag_set: Some(Some(record[2].split('|').map(|s| s.to_string()).collect())),
                tracking_id: Some(Some(record[3].to_string())),
                metadata: Some(Some(
                    record[4].to_string().replace(';', ",").parse().unwrap(),
                )),
                upsert_by_tracking_id: Some(Some(true)),
                ..Default::default()
            };
            chunk_data
        })
        .collect();

    let mut handles = vec![];

    for chunk in chunk_data.chunks(120) {
        let settings = settings.clone();
        let dataset_id = dataset_id.clone();
        let chunk = chunk.to_vec();
        let handle = tokio::spawn(async move {
            let config = Configuration {
                base_path: settings.api_url,
                api_key: Some(ApiKey {
                    prefix: None,
                    key: settings.api_key,
                }),
                ..Default::default()
            };

            let data = CreateChunkParams {
                tr_dataset: dataset_id.clone().unwrap(),
                create_chunk_data: trieve_client::models::CreateChunkData::CreateBatchChunkData(
                    chunk,
                ),
            };

            let result = create_chunk(&config, data)
                .await
                .map_err(|e| DefaultError {
                    message: e.to_string(),
                })?
                .entity
                .unwrap();

            match result {
                trieve_client::apis::chunk_api::CreateChunkSuccess::Status200(_) => Ok(()),
                trieve_client::apis::chunk_api::CreateChunkSuccess::UnknownValue(val) => {
                    Err(DefaultError {
                        message: format!("Error adding seed data: {}", val),
                    })
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await.unwrap().map_err(|e| {
            eprintln!("Error adding seed data: {:?}", e);
        });
    }

    Ok(())
}

async fn add_trieve_mintlify_docs(
    settings: TrieveConfiguration,
    dataset_id: Option<String>,
) -> Result<(), DefaultError> {
    let res = add_json_dataset(
        "https://gist.githubusercontent.com/skeptrunedev/dc34aa54f7810c913794ad045cc767d2/raw/4205cf3ab0dd55fccdc3a336bc26ce6a16b82cf3/trieve-mintlify-docs-chunks.json",
        settings,
        dataset_id,
    )
    .await?;

    Ok(res)
}

async fn add_mintlify_docs(
    settings: TrieveConfiguration,
    dataset_id: Option<String>,
) -> Result<(), DefaultError> {
    let res = add_json_dataset(
        "https://gist.githubusercontent.com/densumesh/0400c4519e55dfcd8d8d2e4a171fc531/raw/df73e08c4173128ba321f506ff763b2bdce4e273/mintlify_chunks.json",
        settings,
        dataset_id,
    )
    .await?;

    Ok(res)
}

async fn add_json_dataset(
    gist_url: &str,
    settings: TrieveConfiguration,
    dataset_id: Option<String>,
) -> Result<(), DefaultError> {
    let chunks_to_create = ureq::get(gist_url).call().map_err(|e| DefaultError {
        message: e.to_string(),
    })?;
    let chunks_to_create: serde_json::Value =
        chunks_to_create.into_json().map_err(|e| DefaultError {
            message: e.to_string(),
        })?;

    let config = Configuration {
        base_path: settings.api_url.clone(),
        api_key: Some(ApiKey {
            prefix: None,
            key: settings.api_key.clone(),
        }),
        ..Default::default()
    };

    let mut group_tracking_ids: HashSet<String> = HashSet::new();

    chunks_to_create
        .as_array()
        .expect("Should always be an array")
        .iter()
        .for_each(|chunk| {
            let chunk = chunk.as_object().expect("Should always be an object");
            let cur_group_tracking_ids = chunk.get("group_tracking_ids").map(|g| {
                g.as_array().map(|a| {
                    a.iter().for_each(|v| {
                        group_tracking_ids.insert(v.as_str().unwrap().to_string());
                    })
                })
            });
        });

    let chunk_datas: Vec<ChunkData> = chunks_to_create
        .as_array()
        .expect("Should always be an array")
        .iter()
        .map(|chunk| {
            let chunk = chunk.as_object().expect("Should always be an object");
            let chunk_data = ChunkData {
                link: Some(chunk["link"].as_str().map(|s| s.to_string())),
                chunk_html: Some(chunk["chunk_html"].as_str().map(|s| s.to_string())),
                metadata: Some(
                    chunk["metadata"]
                        .as_object()
                        .map(|m| m.clone())
                        .map(serde_json::Value::Object),
                ),
                tracking_id: Some(chunk["tracking_id"].as_str().map(|s| s.to_string())),
                tag_set: Some(
                    chunk["tag_set"]
                        .as_str()
                        .map(|s| s.split(',').map(|s| s.to_string()).collect()),
                ),
                group_tracking_ids: chunk.get("group_tracking_ids").map(|g| {
                    g.as_array()
                        .map(|a| a.iter().map(|v| v.as_str().unwrap().to_string()).collect())
                }),
                upsert_by_tracking_id: Some(Some(true)),
                ..Default::default()
            };
            chunk_data
        })
        .collect();

    for tracking_id in group_tracking_ids {
        let group_data = CreateChunkGroupData {
            name: tracking_id.clone(),
            tracking_id: Some(Some(tracking_id.clone())),
            ..Default::default()
        };

        let data = trieve_client::apis::chunk_group_api::CreateChunkGroupParams {
            tr_dataset: dataset_id.clone().unwrap(),
            create_chunk_group_data: group_data,
        };

        let result = create_chunk_group(&config, data)
            .await
            .map_err(|e| DefaultError {
                message: e.to_string(),
            })?
            .entity
            .unwrap();

        match result {
            trieve_client::apis::chunk_group_api::CreateChunkGroupSuccess::Status200(_) => continue,
            trieve_client::apis::chunk_group_api::CreateChunkGroupSuccess::UnknownValue(val) => {
                return Err(DefaultError {
                    message: format!("Error creating group: {}", val),
                });
            }
        }
    }

    let mut handles = vec![];

    for chunks in chunk_datas.chunks(120) {
        let settings = settings.clone();
        let dataset_id = dataset_id.clone();
        let chunks = chunks.to_vec();
        let handle = tokio::spawn(async move {
            let config = Configuration {
                base_path: settings.api_url.clone(),
                api_key: Some(ApiKey {
                    prefix: None,
                    key: settings.api_key.clone(),
                }),
                ..Default::default()
            };

            let data = CreateChunkParams {
                tr_dataset: dataset_id.clone().unwrap(),
                create_chunk_data: trieve_client::models::CreateChunkData::CreateBatchChunkData(
                    chunks,
                ),
            };

            let result = create_chunk(&config, data)
                .await
                .map_err(|e| DefaultError {
                    message: e.to_string(),
                })?
                .entity
                .unwrap();

            match result {
                trieve_client::apis::chunk_api::CreateChunkSuccess::Status200(_) => Ok(()),
                trieve_client::apis::chunk_api::CreateChunkSuccess::UnknownValue(val) => {
                    Err(DefaultError {
                        message: format!("Error adding seed data: {}", val),
                    })
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await.unwrap();
    }

    Ok(())
}

async fn add_philosophize_this_seed_data(
    settings: TrieveConfiguration,
    dataset_id: Option<String>,
) -> Result<(), DefaultError> {
    let groups_to_create = ureq::get("https://gist.githubusercontent.com/densumesh/241be979beb48b05a01591b7ff40ddca/raw/83ad1c0c75c016832368183fc2cb86cb3d7f9c50/philosiphizethis-epLinks.csv").call().map_err(
        |e| DefaultError {
            message: e.to_string(),
        }
    )?;

    let config = Configuration {
        base_path: settings.api_url.clone(),
        api_key: Some(ApiKey {
            prefix: None,
            key: settings.api_key.clone(),
        }),
        ..Default::default()
    };

    let mut group_rdr = csv::Reader::from_reader(groups_to_create.into_reader());

    let group_data: Vec<CreateChunkGroupData> = group_rdr
        .records()
        .map(|record| {
            let record = record.expect("Error reading CSV record");
            CreateChunkGroupData {
                name: record[1].to_string(),
                tracking_id: Some(Some(record[1].to_string())),
                ..Default::default()
            }
        })
        .collect();

    for group in group_data {
        let data = trieve_client::apis::chunk_group_api::CreateChunkGroupParams {
            tr_dataset: dataset_id.clone().unwrap(),
            create_chunk_group_data: group,
        };

        let result = create_chunk_group(&config, data)
            .await
            .map_err(|e| DefaultError {
                message: e.to_string(),
            })?
            .entity
            .unwrap();

        match result {
            trieve_client::apis::chunk_group_api::CreateChunkGroupSuccess::Status200(_) => continue,
            trieve_client::apis::chunk_group_api::CreateChunkGroupSuccess::UnknownValue(val) => {
                return Err(DefaultError {
                    message: format!("Error creating group: {}", val),
                });
            }
        }
    }

    let chunks_to_create = ureq::get("https://gist.githubusercontent.com/densumesh/33f34fa0ca115723b2c25a862a2d2a4b/raw/f282e21f12ccaa2ad10a8fa831d238fa4a8aa1b0/philosiphizethis-chunksToCreate.csv").call().map_err(
        |e| DefaultError {
            message: e.to_string(),
        }
    )?;

    let mut chunk_rdr = ReaderBuilder::new()
        .delimiter(b'|')
        .from_reader(chunks_to_create.into_reader());

    let chunk_data: Vec<ChunkData> = chunk_rdr
        .records()
        .map(|record| {
            let record = record.expect("Error reading CSV record");
            ChunkData {
                group_tracking_ids: Some(Some(vec![record[0].to_string()])),
                tracking_id: Some(Some(record[1].to_string())),
                chunk_html: Some(Some(record[2].to_string())),
                time_stamp: Some(Some(record[3].to_string())),
                link: Some(Some(record[4].to_string())),
                metadata: Some(Some(json!({
                    "episode_number": record[5].to_string(),
                    "episode_title": record[6].to_string(),
                }))),
                upsert_by_tracking_id: Some(Some(true)),
                ..Default::default()
            }
        })
        .collect();

    let mut handles = vec![];

    for chunk in chunk_data.chunks(120) {
        let settings = settings.clone();
        let dataset_id = dataset_id.clone();
        let chunk = chunk.to_vec();
        let handle = tokio::spawn(async move {
            let config = Configuration {
                base_path: settings.api_url,
                api_key: Some(ApiKey {
                    prefix: None,
                    key: settings.api_key,
                }),
                ..Default::default()
            };

            let data = CreateChunkParams {
                tr_dataset: dataset_id.clone().unwrap(),
                create_chunk_data: trieve_client::models::CreateChunkData::CreateBatchChunkData(
                    chunk,
                ),
            };

            let result = create_chunk(&config, data)
                .await
                .map_err(|e| DefaultError {
                    message: e.to_string(),
                })?
                .entity
                .unwrap();

            match result {
                trieve_client::apis::chunk_api::CreateChunkSuccess::Status200(_) => Ok(()),
                trieve_client::apis::chunk_api::CreateChunkSuccess::UnknownValue(val) => {
                    Err(DefaultError {
                        message: format!("Error adding seed data: {:?}", val.as_object()),
                    })
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await.unwrap().map_err(|e| {
            eprintln!("Error adding seed data: {:?}", e);
        });
    }

    Ok(())
}

pub async fn add_seed_data(
    settings: TrieveConfiguration,
    seed_data: AddSeedData,
) -> Result<(), DefaultError> {
    if settings.organization_id.to_string().is_empty() || settings.api_key.is_empty() {
        eprintln!("Please login to the Trieve CLI with your credentials. Run `trieve login` to get started.");
        std::process::exit(1);
    }

    let mut dataset_id = seed_data.dataset_id.clone();

    if dataset_id.is_none() {
        let datasets = get_datasets_from_org(settings.clone())
            .await
            .map_err(|e| {
                eprintln!("Error listing datasets: {}", e.message);
                std::process::exit(1);
            })?
            .iter()
            .map(|d| DatasetAndUsageDTO(d.clone()))
            .collect::<Vec<_>>();

        let ans = Confirm::new("Would you like to create a new dataset?")
            .with_default(true)
            .with_help_message("If you select No, you will be prompted to select an existing dataset to add seed data to.")
            .prompt();

        if ans.unwrap() {
            let create = CreateDataset { name: None };

            let dataset = create_trieve_dataset(settings.clone(), create).await?;
            dataset_id = Some(dataset.id.to_string());
        } else {
            let selected_dataset =
                inquire::Select::new("Select a dataset to add seed data to:", datasets)
                    .prompt()
                    .unwrap();
            dataset_id = Some(selected_dataset.0.dataset.id.to_string());
        }
    }

    let selected_example = inquire::Select::new(
        "Select an example dataset to add:",
        vec![
            "YC Companies",
            "PhilosiphizeThis",
            "Trieve Docs",
            "Mintlify Docs",
        ],
    )
    .prompt()
    .unwrap();

    println!(
        "Adding seed data to dataset: {}",
        dataset_id.clone().unwrap()
    );

    match selected_example {
        "YC Companies" => add_yc_companies_seed_data(settings.clone(), dataset_id).await?,
        "PhilosiphizeThis" => add_philosophize_this_seed_data(settings.clone(), dataset_id).await?,
        "Trieve Docs" => add_trieve_mintlify_docs(settings.clone(), dataset_id).await?,
        "Mintlify Docs" => add_mintlify_docs(settings.clone(), dataset_id).await?,
        _ => {
            eprintln!("Invalid example dataset selected: {}", selected_example);
            std::process::exit(1);
        }
    }

    println!("Example dataset added successfully!");
    Ok(())
}
