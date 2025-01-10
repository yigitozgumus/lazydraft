# LazyDraft

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[![Version](https://img.shields.io/github/v/release/yigitozgumus/lazydraft)](https://github.com/yigitozgumus/lazydraft/releases)

LazyDraft is a powerful tool designed to streamline the management of markdown blog projects within your Obsidian workflow. It enables you to transfer projects to static site generators (like Hugo or Jekyll) seamlessly, making blogging effortless for developers and content creators alike.

## Features

- **Efficient Blog Management**: Organize your markdown files and blog projects in a structured way.
- **Static Site Integration**: Automatically transfer and format content for popular static site generators.
- **CLI-Based Workflow**: Perform all tasks via simple commands in the terminal.
- **Cross-Platform Support**: Available on macOS, Linux, and Windows.

## Motivation

I usually write my content in Obsidian. Keeping everything in there makes sense for me, but when I want to move something out of Obsidian to my website, I need to:

- Modify the content of the writing if I have any images or assets.
- Find the related asset files and copy them as well.

Doing this once is manageable, but if I don't like something and change it, I now need to update it in multiple places.

**LazyDraft** simplifies this process, automating the steps and reducing manual intervention.

## Installation

### Using Homebrew (macOS/Linux)

```bash
brew install yigitozgumus/formulae/lazydraft
```

### Using Precompiled Binaries

Visit the Releases page

1.  Go to https://github.com/yigitozgumus/lazydraft/releases
2.  Download the binary for your operating system:
    - lazydraft-linux-amd64.tar.gz
    - lazydraft-macos-amd64.tar.gz
    - lazydraft-windows-amd64.zip
      Extract the binary and add it to your PATH:
      Example for Linux/macOS:

```bash
tar -xzf lazydraft-linux-amd64.tar.gz
mv lazydraft /usr/local/bin/
```

### From Source

```bash
# Clone the repository:
git clone https://github.com/yigitozgumus/lazydraft.git
cd lazydraft

# Build the binary:
cargo build --release

# Move the binary to your PATH:
mv target/release/lazydraft /usr/local/bin/
```

## Usage

LazyDraft provides three commands:

### 1. `status`

Checks the current status of your notes and identifies which ones are ready for publishing.

```bash
    lazydraft status
```

### 2. `stage`

Prepares and transfers notes and assets to the target directory for publishing.

```bash
lazydraft stage
```

Options:

- --continuous: Watches for changes in your Obsidian vault and stages content automatically.

```bash
lazydraft stage --continuous
```

### 3. `config`

Creates or validates the configuration file lazydraft.json.
On the first run, it generates an empty configuration file.
```bash
lazydraft config
```

Refer to the Configuration section for more details on setting up the `lazydraft.json` file.

You also  can find more information in my accompanying [writing series](https://www.yigitozgumus.com/series/building-a-cli-in-rust/).


## Configuration

LazyDraft uses a JSON configuration file (lazydraft.json) to define project-specific settings.

### Example Configuration

```json
{
  "source_dir": "/source",
  "source_asset_dir": "/source_asset",
  "target_dir": "/target_dir",
  "target_asset_dir": "/target_asset",
  "target_asset_prefix": "./assets",
  "yaml_asset_prefix": "assetPrefix",
  "sanitize_frontmatter": true,
  "auto_add_cover_img": true,
  "remove_draft_on_stage": true,
  "add_date_prefix":false,
  "remove_wikilinks": true
}

```

### How to Edit Configuration

Run the config command to interactively validate or update the configuration.

```bash
lazydraft config
```

This command will guide you through updating the lazydraft.json file.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## Support

If you encounter issues or have feature requests, please [open an issue](https://github.com/yigitozgumus/lazydraft/issues).

Contributions are welcome! Refer to the `CONTRIBUTING.md` file for guidelines.
