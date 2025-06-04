# LazyDraft

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[![Version](https://img.shields.io/github/v/release/yigitozgumus/lazydraft)](https://github.com/yigitozgumus/lazydraft/releases)

LazyDraft is a powerful tool designed to streamline the management of markdown blog projects within your Obsidian workflow. It enables you to transfer projects to static site generators (like Hugo or Jekyll) seamlessly, making blogging effortless for developers and content creators alike.

## Features

- **Multi-Project Management**: Organize and switch between multiple blog projects with ease.
- **Efficient Blog Management**: Organize your markdown files and blog projects in a structured way.
- **Static Site Integration**: Automatically transfer and format content for popular static site generators.
- **CLI-Based Workflow**: Perform all tasks via simple commands in the terminal.
- **Cross-Platform Support**: Available on macOS, Linux, and Windows.
- **Legacy Migration**: Automatically migrates existing configurations to the new project system.

## Motivation

I usually write my content in Obsidian. Keeping everything in there makes sense for me, but when I want to move something out of Obsidian to my website, I need to:

- Modify the content of the writing if I have any images or assets.
- Find the related asset files and copy them as well.

Doing this once is manageable, but if I don't like something and change it, I now need to update it in multiple places.

**LazyDraft** simplifies this process, automating the steps and reducing manual intervention. With the new multi-project support, you can manage multiple blogs, documentation sites, or content projects simultaneously.

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

LazyDraft provides commands for both project management and content operations:

### Project Management

#### Creating and Managing Projects

```bash
# Create a new project
lazydraft project create my-blog "Personal blog content"

# List all projects (shows active project with ●)
lazydraft project list

# Switch to a different project
lazydraft project switch my-blog

# Get information about current or specific project
lazydraft project info
lazydraft project info my-blog

# Rename a project
lazydraft project rename old-name new-name

# Delete a project (cannot delete active project)
lazydraft project delete old-project
```

### Content Operations

#### 1. `status`

Checks the current status of your notes and identifies which ones are ready for publishing.

```bash
# Check status of active project
lazydraft status

# Check status of specific project
lazydraft status --project my-blog
```

#### 2. `stage`

Prepares and transfers notes and assets to the target directory for publishing.

```bash
# Stage content from active project
lazydraft stage

# Stage content from specific project
lazydraft stage --project my-blog

# Continuous monitoring (watches for changes)
lazydraft stage --continuous
lazydraft stage --continuous --project my-blog
```

#### 3. `config`

Creates or validates the configuration file for projects.

```bash
# Edit active project configuration
lazydraft config --edit

# Edit specific project configuration
lazydraft config --edit --project my-blog

# Show configuration help
lazydraft config --info
```

## Configuration

LazyDraft uses TOML configuration files to define project-specific settings. Each project has its own configuration file stored in `~/.config/lazydraft/projects/`.

### Project Structure

```
~/.config/lazydraft/
├── active_project.toml          # Tracks current active project
└── projects/
    ├── my-blog.toml            # Individual project configs
    ├── work-docs.toml
    └── portfolio.toml
```

### Example Project Configuration

```toml
name = "my-blog"
description = "Personal blog content"
created_at = "2025-06-04T09:37:50.031739+00:00"
last_used = "2025-06-04T09:37:50.031739+00:00"

# Content processing settings
source_dir = "/Users/username/obsidian/blog"
source_asset_dir = "/Users/username/obsidian/blog/assets"
target_dir = "/Users/username/sites/my-blog/content"
target_asset_dir = "/Users/username/sites/my-blog/static/assets"
target_asset_prefix = "./assets"
yaml_asset_prefix = "assetPrefix"

# Processing options
sanitize_frontmatter = true
auto_add_cover_img = true
auto_add_hero_img = false
remove_draft_on_stage = true
add_date_prefix = false
remove_wikilinks = true
trim_tags = false
use_mdx_format = false
```

### Configuration Options

- `source_dir`: Directory where source files are located
- `source_asset_dir`: Directory where assets for the source are stored
- `target_dir`: Directory where output files are generated
- `target_asset_dir`: Directory where output assets are stored
- `target_asset_prefix`: Prefix for asset links in the generated files
- `target_hero_image_prefix`: Prefix for hero image links in the output
- `yaml_asset_prefix`: Prefix for assets referenced in YAML frontmatter
- `sanitize_frontmatter`: If true, removes empty fields from the frontmatter
- `auto_add_cover_img`: Automatically adds a cover image to the frontmatter
- `auto_add_hero_img`: Automatically adds a hero image to the frontmatter
- `remove_draft_on_stage`: Sets the 'draft' flag to false when staging
- `add_date_prefix`: Adds a date prefix to the file name
- `remove_wikilinks`: Converts wiki-style links to plain markdown links
- `trim_tags`: Strips a specified prefix from tags in frontmatter
- `tag_prefix`: The prefix to strip from tags when 'trim_tags' is enabled
- `use_mdx_format`: If true, saves output files with the `.mdx` extension instead of `.md`

### Legacy Migration

If you have an existing `lazydraft.toml` or `lazydraft.json` file, LazyDraft will automatically migrate it to a new project called "default" when you first run any command. Your existing workflow will continue to work without any changes.

## Examples

### Multi-Project Workflow

```bash
# Set up multiple projects
lazydraft project create personal-blog "My personal writing"
lazydraft project create work-docs "Technical documentation"
lazydraft project create portfolio "Portfolio content"

# Configure each project
lazydraft config --edit --project personal-blog
lazydraft config --edit --project work-docs

# Work with different projects
lazydraft project switch personal-blog
lazydraft status
lazydraft stage

lazydraft project switch work-docs
lazydraft stage --continuous

# Check project status
lazydraft project list
```

### Single Project Migration

If you're upgrading from an older version:

```bash
# Your existing config is automatically migrated
lazydraft project list
# Shows: ● default (Migrated from legacy configuration)

# Rename if desired
lazydraft project rename default my-blog

# Continue working as before
lazydraft status
lazydraft stage
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## Support

If you encounter issues or have feature requests, please [open an issue](https://github.com/yigitozgumus/lazydraft/issues).

Contributions are welcome! Refer to the `CONTRIBUTING.md` file for guidelines.
