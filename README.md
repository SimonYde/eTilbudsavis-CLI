# eTilbudsavis-CLI

A CLI interface for [etilbudsavis.dk](https://etilbudsavis.dk), written in Rust. This tool allows users to interact with the eTilbudsavis service to manage favorite dealers and retrieve offers.

## Prerequisites

- Rust and Cargo installed on your system (if using Cargo for installation).
- Nix package manager (if using the Nix flake).

## Installation

You can install the tool using Cargo or via the provided Nix flake. Follow the instructions below based on your preferred method.

### Using Cargo

1. Ensure you have Rust and Cargo installed. If not, install them from the [official Rust website](https://www.rust-lang.org/tools/install).

2. Install the tool via Cargo:

   ```bash
   cargo install etilbudsavis-cli
   ```

   This will download and build the tool from the source repository.

### Using Nix Flake

If you have Nix installed, you can use the provided Nix flake for a reproducible build.

1. Running using Nix:

   ```bash
   nix run github:SimonYde/etilbudsavis-cli
   ```

2. Add as input to a nix flake
   Refer to the [Nix documentation](https://nixos.org/manual/nix/stable/) for more details on using flakes.

## Usage

This tool provides a command-line interface to interact with the eTilbudsavis API. To retrieve offers, you must first add at least one dealer to your favorites. Additionally, specify an output format using the ```-f``` or ```--format``` flag; available options are ```table```, ```json```, or ```rss```.

### General Usage

The basic command structure is:

```
etilbudsavis-cli {flags} ...(search)
```

- **Flags**:
  - ```-f, --format <string>```: Specifies the output format. Options: ```table```, ```json```, ```rss```.
  - ```-d, --dealer```: Filters searches by a specific dealer.
  - ```--generate <string>```: Generate shell completions. Check help for available options.
  - ```-h, --help```: Displays help information for the command or subcommand.
  - ```-V, --version```: Prints the version of the tool.

- **Parameters**:
  - ```...search <string>```: Provides search terms for queries (used in the main command).

### Subcommands

Use the following subcommands to manage dealers and perform actions:

- ```add```: Add a dealer to your favorites.
  - Example: ```etilbudsavis-cli add "Netto"```
  - Note: You must specify a dealer to add it.

- ```dealers```: List all available dealers.
  - Example: ```etilbudsavis-cli dealers -f json```
  - This will output the list in the specified format.

- ```favorites```: List your currently set favorite dealers.
  - Example: ```etilbudsavis-cli favorites -f table```

- ```help```: Print this help message or help for a specific subcommand.
  - Example: ```etilbudsavis-cli help add```

- ```remove```: Remove a dealer from your favorites.
  - Example: ```etilbudsavis-cli remove "Netto"```

### Examples

Here are some practical examples to get started:

1. List available dealers in JSON format:
   ```
   etilbudsavis-cli dealers -f json
   ```

2. Add a dealer to favorites and then list favorites:
   ```
   etilbudsavis-cli add "Netto"
   etilbudsavis-cli favorites -f table
   ```

3. Perform a search (after adding favorites):
   ```
   etilbudsavis-cli -f rss "Pasta"
   ```
   - This assumes you have favorites added. If not, the search may not return results.

4. Remove a dealer:
   ```
   etilbudsavis-cli remove "Netto"
   ```

## Additional Notes

- **Output Formats**: Always specify a format with ```-f``` or ```--format``` to receive output. For instance, use ```json``` for machine-readable data or ```rss``` for feed integration.
- **Troubleshooting**: If you encounter issues, check the [GitHub repository](https://github.com/SimonYde/eTilbudsavis-CLI) for updates or open an issue for support.

For more details on usage, run ```etilbudsavis-cli help```.
