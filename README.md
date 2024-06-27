<p align="center">
    <h1 align="center">
      <picture>
        <source media="(prefers-color-scheme: light)" srcset="https://github.com/privacy-scaling-explorations/zk-kit/assets/11427903/f691c48c-021f-485d-89ef-9ddc8ba74787">
        <source media="(prefers-color-scheme: dark)" srcset="https://github.com/privacy-scaling-explorations/zk-kit/assets/11427903/f43f4403-846a-48b4-a1fa-0ab234c225e5">
        <img width="250" alt="ZK-Kit logo" src="https://github.com/privacy-scaling-explorations/zk-kit/assets/11427903/f691c48c-021f-485d-89ef-9ddc8ba74787">
      </picture>
      <sub>Rust</sub>
    </h1>
</p>

<p align="center">
    <a href="https://github.com/privacy-scaling-explorations" target="_blank">
        <img src="https://img.shields.io/badge/project-PSE-blue.svg?style=flat-square">
    </a>
    <a href="https://github.com/privacy-scaling-explorations/zk-kit.rust/blob/main/LICENSE">
        <img alt="Github license" src="https://img.shields.io/github/license/privacy-scaling-explorations/zk-kit.rust.svg?style=flat-square">
    </a>
    <a href="https://github.com/privacy-scaling-explorations/zk-kit.rust/actions?query=workflow%3Amain">
        <img alt="GitHub Main Workflow" src="https://img.shields.io/github/actions/workflow/status/privacy-scaling-explorations/zk-kit.rust/main.yml?branch=main&label=main&style=flat-square&logo=github">
    </a>
</p>

<div align="center">
    <h4>
        <a href="/CONTRIBUTING.md">
            👥 Contributing
        </a>
        <span>&nbsp;&nbsp;|&nbsp;&nbsp;</span>
        <a href="/CODE_OF_CONDUCT.md">
            🤝 Code of conduct
        </a>
        <span>&nbsp;&nbsp;|&nbsp;&nbsp;</span>
        <a href="https://github.com/privacy-scaling-explorations/zk-kit.rust/issues/new/choose">
            🔎 Issues
        </a>
        <span>&nbsp;&nbsp;|&nbsp;&nbsp;</span>
        <a href="https://appliedzkp.org/discord">
            🗣️ Chat &amp; Support
        </a>
    </h4>
</div>

| ZK-Kit is a set of libraries (algorithms or utility functions) that can be reused in different projects and zero-knowledge protocols, making it easier for developers to access user-friendly, tested, and documented code for common tasks. ZK-Kit provides different repositories for each language - this one contains Rust code only. |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |

## 🗂️ Repositories

-   Javascript: https://github.com/privacy-scaling-explorations/zk-kit
-   Rust: https://github.com/privacy-scaling-explorations/zk-kit.rust
-   Solidity: https://github.com/privacy-scaling-explorations/zk-kit.solidity
-   Circom: https://github.com/privacy-scaling-explorations/zk-kit.circom
-   Noir: https://github.com/privacy-scaling-explorations/zk-kit.noir

## 📦 Crates

<table>
    <th>Package</th>
    <th>Version</th>
    <th>Downloads</th>
    <th>Audited</th>
    <tbody>
        <tr>
            <td>
                <a href="https://github.com/privacy-scaling-explorations/zk-kit.rust/tree/main/crates/example">
                    example
                </a>
            </td>
            <td>
                <!-- Crate version -->
                <a href="https://docs.rs/example">
                    <img src="https://img.shields.io/crates/d/example?style=flat-square" alt="Crate version" />
                </a>
            </td>
            <td>
                <!-- Crate downloads -->
                <a href="https://docs.rs/example">
                    <img src="https://img.shields.io/crates/v/example?style=flat-square" alt="Crate downloads" />
                </a>
            </td>
            <td>
                ❌
            </td>
        </tr>
    <tbody>
</table>

## 👥 Ways to contribute

-   🔧 Work on [open issues](https://github.com/privacy-scaling-explorations/zk-kit.rust/contribute)
-   📦 Suggest new [packages](https://github.com/privacy-scaling-explorations/zk-kit.rust/issues/new?assignees=&labels=feature+%3Arocket%3A&template=---package.md&title=)
-   🚀 Share ideas for new [features](https://github.com/privacy-scaling-explorations/zk-kit.rust/issues/new?assignees=&labels=feature+%3Arocket%3A&template=---feature.md&title=)
-   🐛 Create a report if you find any [bugs](https://github.com/privacy-scaling-explorations/zk-kit.rust/issues/new?assignees=&labels=bug+%F0%9F%90%9B&template=---bug.md&title=) in the code

## 🛠 Install

Clone this repository:

```bash
git clone https://github.com/privacy-scaling-explorations/zk-kit.rust.git
```

and install the dependencies:

```bash
cd zk-kit.rust && cargo build
```

## 📜 Usage

### Code quality and formatting

Run [rustfmt](https://github.com/rust-lang/rustfmt) to check formatting rules:

```bash
cargo fmt -- --check
```

or automatically format the code:

```bash
cargo fmt
```

Run [Clippy](https://github.com/rust-lang/rust-clippy) to analyze the code and catch bugs:

```bash
cargo clippy --workspace --all-targets
```

or automatically apply suggestions:

```bash
cargo clippy --workspace --fix
```

### Conventional commits

ZK-Kit uses [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/). We therefore suggest using tools such as [cocogitto](https://docs.cocogitto.io).

### Testing

Test the code:

```bash
cargo test --workspace --all-targets
```

### Build

Build crates:

```bash
cargo build --workspace --all-targets
```
