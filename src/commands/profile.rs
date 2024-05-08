use tabled::{builder::Builder, settings::Style};

use crate::{commands::configure::TrieveProfile, DeleteProfile, SwitchProfile};

use super::configure::TrieveProfileInner;

pub fn switch_profile(
    profile_data: SwitchProfile,
    profiles: Vec<TrieveProfileInner>,
) -> Result<(), Box<dyn std::error::Error>> {
    let profile_name = profile_data.profile_name.unwrap_or_else(|| {
        let profile_name = inquire::Select::new(
            "Select a profile to switch to:",
            profiles.iter().map(|p| p.name.clone()).collect(),
        )
        .with_starting_cursor(profiles.iter().position(|p| p.selected).unwrap_or(0))
        .prompt()
        .unwrap();
        profile_name
    });

    profiles
        .iter()
        .find(|p| p.name == profile_name)
        .ok_or_else(|| {
            eprintln!("Profile '{}' not found.", profile_name);
            std::process::exit(1);
        })
        .unwrap();

    let profiles = profiles
        .iter()
        .map(|p| {
            if p.name == profile_name {
                TrieveProfileInner {
                    name: p.name.clone(),
                    selected: true,
                    settings: p.settings.clone(),
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

    println!("Switched to profile '{}'.", profile_name);

    Ok(())
}

pub fn list_profiles(
    mut profiles: Vec<TrieveProfileInner>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::default();

    profiles.sort_by(|a, b| b.selected.cmp(&a.selected));

    builder.push_record(["Name", "API Url", "Selected"]);

    for profile in profiles {
        builder.push_record([
            profile.name,
            profile.settings.api_url,
            if profile.selected {
                "✔".to_owned()
            } else {
                "".to_owned()
            },
        ]);
    }

    let table = builder.build().with(Style::rounded()).to_string();
    println!("Profiles:");
    println!("{}", table);

    Ok(())
}

pub fn delete_profile(
    profile_data: DeleteProfile,
    profiles: Vec<TrieveProfileInner>,
) -> Result<(), Box<dyn std::error::Error>> {
    let profile_name = profile_data.profile_name.unwrap_or_else(|| {
        let profile_name = inquire::Select::new(
            "Select a profile to delete:",
            profiles.iter().map(|p| p.name.clone()).collect(),
        )
        .prompt()
        .unwrap();
        profile_name
    });

    let profile = profiles
        .iter()
        .find(|p| p.name == profile_name)
        .ok_or_else(|| {
            eprintln!("Profile '{}' not found.", profile_name);
            std::process::exit(1);
        })
        .unwrap();

    let mut profiles = profiles
        .iter()
        .filter(|p| p.name != profile_name)
        .map(|p| p.clone())
        .collect::<Vec<TrieveProfileInner>>();

    if profile.selected {
        if profiles.is_empty() {
            eprintln!("Cannot delete the last profile.");
            std::process::exit(1);
        }

        profiles[0].selected = true;
    }

    confy::store("trieve", "profiles", TrieveProfile { inner: profiles })
        .map_err(|e| {
            eprintln!("Error saving configuration: {:?}", e);
            std::process::exit(1);
        })
        .unwrap();

    println!("Deleted profile '{}'.", profile_name);

    Ok(())
}
