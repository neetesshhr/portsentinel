# Contributing to PortSentinel

First off, thank you for considering contributing to PortSentinel! It's people like you that make PortSentinel such a great tool. We welcome contributions from everyone, regardless of your skill level.

## 👋 Welcome!

We want to make contributing to this project as easy and transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## 🛠️ How to Contribute

### 1. Fork & Clone
Fork the repository on GitHub and clone it to your local machine:

```bash
git clone [https://github.com/YOUR_USERNAME/PortSentinel.git](https://github.com/YOUR_USERNAME/PortSentinel.git)
cd PortSentinel
```

### 2. Create a branch
Create a new branch for your feature or fix. Please use a descriptive name:
```bash
git checkout -b feature/add-docker-stats
# or
git checkout -b fix/login-overflow-bug
```

### 3. Make Your Changes
Write your code!
- **Rust Code:** Please run `cargo fmt` to ensure your code matches our style guidelines.
- **Frontend:** We use Tailwind CSS and vanilla JS. Keep it simple and dependency-free where possible.

### 4. Test Your Changes
Ensure everything builds and runs:
```bash
# Terminal 1: Run the Agent
cargo run -p port_sentinel_agent

# Terminal 2: Run the Master
cd master && cargo run
```

### 5. Submit a Pull Request (PR)

- Push your branch to your fork and submit a Pull Request to our `main` branch.

- Title: Clear and concise (e.g., "Added Docker container restart button").

- Description: Explain what you changed and why.

- Screenshots: If you changed the UI, please include a screenshot!

## 🐛 Reporting Bugs
If you find a bug, please create an Issue on GitHub. Include:

- Your OS (e.g., Ubuntu 22.04, macOS M2).

- Steps to reproduce the bug.

- Expected vs. actual behavior.

## 💡 Feature Requests

Have an idea? We'd love to hear it! Open an Issue with the tag `enhancement` and tell us:

- What problem you are trying to solve.

- How you imagine the solution working.

## 📜 Code of Conduct
We are committed to providing a friendly, safe, and welcoming environment for all. Please be respectful and considerate in your communication.

## 🤝 Need Help?
If you have questions, feel free to reach out in the Discussions tab .

Thank you for helping us build the fastest monitoring tool on the planet! 🚀🚀
