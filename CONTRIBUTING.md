# Contributing to LazyDraft

Thank you for your interest in contributing to LazyDraft! Your help is highly appreciated, whether it's reporting issues, suggesting features, or submitting code improvements.

## Table of Contents

- [Contributing to LazyDraft](#contributing-to-lazydraft)
  - [Table of Contents](#table-of-contents)
  - [Getting Started](#getting-started)
    - [Steps to Set Up the Project](#steps-to-set-up-the-project)
  - [Reporting Issues](#reporting-issues)
  - [Suggesting Features](#suggesting-features)
  - [Submitting Changes](#submitting-changes)
    - [Steps to Submit a Pull Request](#steps-to-submit-a-pull-request)
  - [Code Guidelines](#code-guidelines)
    - [Coding Standards](#coding-standards)
    - [Tests](#tests)
    - [Commit Messages](#commit-messages)
  - [Need Help?](#need-help)

---

## Getting Started

To get started, make sure you have the following installed on your system:

- [Rust](https://www.rust-lang.org/tools/install)
- [Git](https://git-scm.com/)

### Steps to Set Up the Project

1. **Fork the Repository**:  
   Click the "Fork" button at the top-right corner of the repository page.

2. **Clone Your Fork**:  
   Clone the repository to your local machine:

   ```bash
   git clone https://github.com/<your-username>/lazydraft.git
   cd lazydraft
   ```

3. **Create a New Branch**:  
   Create a feature or bugfix branch for your work:

   ```bash
   git checkout -b feature/your-feature-name
   ```

4. **Build the Project**:  
   Build and test the project to ensure everything is set up correctly:
   ```bash
   cargo build --release
   cargo test
   ```

---

## Reporting Issues

If you find a bug or have a question, please open an issue on the [GitHub Issues](https://github.com/yigitozgumus/lazydraft/issues) page. Make sure to include:

- A clear description of the issue.
- Steps to reproduce (if applicable).
- Relevant logs, screenshots, or error messages.

---

## Suggesting Features

We welcome feature suggestions! When suggesting a feature:

- **Check Existing Issues**: Ensure the feature hasn’t already been requested.
- **Provide Details**: Describe the feature, its purpose, and how it benefits users.

Submit feature requests via the [GitHub Issues](https://github.com/yigitozgumus/lazydraft/issues) page.

---

## Submitting Changes

### Steps to Submit a Pull Request

1. **Make Your Changes**:  
   Develop your feature or bugfix on your branch. Ensure all tests pass:

   ```bash
   cargo test
   ```

2. **Commit Your Changes**:  
   Write a meaningful commit message:

   ```bash
   git add .
   git commit -m "Add detailed description of your change"
   ```

3. **Push Your Branch**:  
   Push your branch to your fork:

   ```bash
   git push origin feature/your-feature-name
   ```

4. **Open a Pull Request**:  
   Go to the main LazyDraft repository and click **New Pull Request**. Select your branch and describe your changes.

---

## Code Guidelines

To maintain consistency and quality, please adhere to the following guidelines:

### Coding Standards

- Follow [Rust's official style guidelines](https://doc.rust-lang.org/1.0.0/style/).
- Use `cargo fmt` to format your code.
- Run `cargo clippy` to lint your code.

### Tests

- Write tests for new features and bug fixes.
- Run all tests before submitting:
  ```bash
  cargo test
  ```

### Commit Messages

- Use descriptive commit messages.
- Example: `Fix panic on invalid config file path` or `Add feature to export markdown files`.

---

## Need Help?

If you have questions about the contribution process or the project, feel free to:

- Open a [GitHub Discussion](https://github.com/yigitozgumus/lazydraft/discussions).
- Reach out to the repository maintainer.

Thank you for contributing to LazyDraft!
