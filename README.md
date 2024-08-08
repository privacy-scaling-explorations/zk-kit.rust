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
        <img alt="GitHub Main Workflow" src="https://img.shields.io/github/actions/workflow/status/privacy-scaling-explorations/zk-kit.rust/main.yml?branch=main&label=tests&style=flat-square&logo=github">
    </a>
    <a href='https://coveralls.io/github/privacy-scaling-explorations/zk-kit.rust?branch=main'>
        <img src='https://coveralls.io/repos/github/privacy-scaling-explorations/zk-kit.rust/badge.svg?branch=main' alt='Coverage Status' />
    </a>
</p>

<div align="center">
    <h4>
        <a href="/CONTRIBUTING.md">
            👥 Contributing
        </a>
        <span>&nbsp;&nbsp;|&nbsp;&nbsp;</span>
        <a href="/CODE_OF_CONDUCT.md">
            🤝 Code of Conduct
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

- **JavaScript:** [zk-kit](https://github.com/privacy-scaling-explorations/zk-kit)
- **Rust:** [zk-kit.rust](https://github.com/privacy-scaling-explorations/zk-kit.rust)
- **Solidity:** [zk-kit.solidity](https://github.com/privacy-scaling-explorations/zk-kit.solidity)
- **Circom:** [zk-kit.circom](https://github.com/privacy-scaling-explorations/zk-kit.circom)
- **Noir:** [zk-kit.noir](https://github.com/privacy-scaling-explorations/zk-kit.noir)

## 📦 Crates

<table>
    <thead>
        <tr>
            <th>Package</th>
            <th>Version</th>
            <th>Downloads</th>
            <th>Audited</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>
                <a href="https://github.com/privacy-scaling-explorations/zk-kit.rust/tree/main/crates/imt">
                    imt
                </a>
            </td>
            <td>
                <a href="https://privacy-scaling-explorations.github.io/zk-kit.rust/zk_kit_imt">
                    <img src="https://img.shields.io/crates/v/zk-kit-imt?style=flat-square" alt="Crate version" />
                </a>
            </td>
            <td>
                <a href="https://privacy-scaling-explorations.github.io/zk-kit.rust/zk_kit_imt">
                    <img src="https://img.shields.io/crates/d/zk-kit-imt?style=flat-square" alt="Crate downloads" />
                </a>
            </td>
            <td>
                ❌
            </td>
        </tr>
        <tr>
            <td>
                <a href="https://github.com/privacy-scaling-explorations/zk-kit.rust/tree/main/crates/smt">
                    smt
                </a>
            </td>
            <td>
                <a href="https://privacy-scaling-explorations.github.io/zk-kit.rust/zk_kit_smt">
                    <img src="https://img.shields.io/crates/v/zk-kit-smt?style=flat-square" alt="Crate version" />
                </a>
            </td>
            <td>
                <a href="https://privacy-scaling-explorations.github.io/zk-kit.rust/zk_kit_smt">
                    <img src="https://img.shields.io/crates/d/zk-kit-smt?style=flat-square" alt="Crate downloads" />
                </a>
            </td>
            <td>
                ❌
            </td>
        </tr>
    <tbody>
</table>

## 👥 Ways to Contribute

- 🔧 Work on [open issues](https://github.com/privacy-scaling-explorations/zk-kit.rust/contribute)
- 📦 Suggest new [crates](https://github.com/privacy-scaling-explorations/zk-kit.rust/issues/new?assignees=&labels=feature+%3Arocket%3A&template=---crate.md&title=)
- 🚀 Share ideas for new [features](https://github.com/privacy-scaling-explorations/zk-kit.rust/issues/new?assignees=&labels=feature+%3Arocket%3A&template=---feature.md&title=)
- 🐛 Create a report if you find any [bugs](https://github.com/privacy-scaling-explorations/zk-kit.rust/issues/new?assignees=&labels=bug+%F0%9F%90%9B&template=---bug.md&title=) in the code

## 🛠 Setup

```bash
git clone https://github.com/privacy-scaling-explorations/zk-kit.rust.git && make setup
```

This will clone this repository and install the required development dependencies locally.

## 📜 Usage

You can view the available tasks with:

```commandline
make
```

### Code Quality and Formatting

You can proof your code for consistency with our formatting rules:

```commandline
make check
```

or automatically format the code (performed automatically in a `pre-commit` hook):

```commandline
make fmt
```

To lint and analyze the code for potential bugs:

```commandline
make lint
```

or automatically apply lint fix suggestions:

```commandline
make fix
```

### Conventional commits

ZK-Kit uses [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/).\
Compliance with these rules is enforced with a `commit-msg` git hook, which was set up when you ran `make setup`.

### Testing

To test the code:

```commandline
make test
```

### Build

To build crates:

```commandline
make build
```
