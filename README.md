# Trieve CLI - README

## Overview

Trieve CLI is a command-line interface (CLI) for interacting with the Trieve Search Product. It allows users to configure profiles, manage API keys, handle datasets, and interact with organizations directly from the command line.

## Installation

To install the Trieve CLI, you need to have Rust installed on your machine. If you don't already have it installed, you can install it with:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then, you can install the CLI from Cargo:

```sh
cargo install trieve
```

After building the project, you can run the CLI using the `trieve` command.

## Usage

To use the Trieve CLI, you need to configure it with your credentials first.

```sh
trieve login
```

Then, you can use any of the available commands to interact with the Trieve service.

## Setting Configuration via Environment Variables

You can configure the Trieve CLI using environment variables. This is especially useful for CI environments where you want to avoid interactive configuration.

To configure the Trieve CLI using environment variables, set the following environment variables and run the desired command:

- **TRIEVE_NO_PROFILE=true**
- **TRIEVE_API_KEY=your_api_key**
- **TRIEVE_ORG_ID=your_organization_id**
- **TRIEVE_API_URL=your_api_url** (optional, defaults to https://api.trieve.ai)

#### Example:

```sh
  TRIEVE_NO_PROFILE=true TRIEVE_API_KEY=api_key TRIEVE_ORG_ID=org_id trieve dataset list
```

With this configuration, you can skip the trieve login step and directly use the CLI commands.

## Features

### General

- **Profile Management**: Switch, delete, and list profiles for different configurations.
- **API Key Management**: Generate new API keys for accessing the Trieve service.
- **Dataset Management**: Create, list, delete, and add seed data to datasets.
- **Organization Management**: Switch between different organizations.

### Commands

#### General Structure

```sh
trieve <command> [subcommand] [flags]
```

NOTE: All of these commands are interactive and will work even without passing in the flags.

#### Commands and Subcommands

1. **Login**

   ```sh
   trieve login --api-key <API_KEY> [--api-url <API_URL>] [--profile-name <PROFILE_NAME>]
   ```

   Configures the Trieve CLI with your API key.

2. **Dataset**

   ```sh
   trieve dataset <subcommand> [flags]
   ```

   - **Create**

     ```sh
     trieve dataset create --name <DATASET_NAME>
     ```

     Creates a dataset in the Trieve service.

   - **List**

     ```sh
     trieve dataset list
     ```

     Lists all datasets in the Trieve service.

   - **Delete**

     ```sh
     trieve dataset delete --dataset-id <DATASET_ID>
     ```

     Deletes a dataset in the Trieve service.

   - **Example (Add Seed Data)**

     ```sh
     trieve dataset example --dataset-id <DATASET_ID>
     ```

     Adds seed data to a dataset in the Trieve service.

3. **API Key**

   ```sh
   trieve apikey <subcommand> [flags]
   ```

   - **Generate**

     ```sh
     trieve apikey generate --name <API_KEY_NAME> --role <API_KEY_ROLE>
     ```

     Generates a new API key.

4. **Profile**

   ```sh
   trieve profile <subcommand> [flags]
   ```

   - **Switch**

     ```sh
     trieve profile switch --profile-name <PROFILE_NAME>
     ```

     Switches to a different profile.

   - **Delete**

     ```sh
     trieve profile delete --profile-name <PROFILE_NAME>
     ```

     Deletes a profile.

   - **List**

     ```sh
     trieve profile list
     ```

     Lists all profiles.

5. **Organization**

   ```sh
   trieve organization <subcommand> [flags]
   ```

   - **Switch**

     ```sh
     trieve organization switch --organization-id <ORGANIZATION_ID>
     ```

     Switches to a different organization.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Contact

For any questions or issues, please open an issue on the GitHub repository or contact the maintainers.

---

This README provides a basic overview of the Trieve CLI and its features. For detailed usage and examples, please refer to the command-specific help by running `trieve <command> --help`.
