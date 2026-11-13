# Contributing to Solana-Bonding-Curve-Launchpad

Thank you for your interest in contributing to **Solana-Bonding-Curve-Launchpad**. This repository implements a production-grade constant-product bonding curve token factory and Raydium CPMM migration engine on Solana using Anchor Rust.

## Guidelines

1. **Rust & Anchor Standards**:
   - Write clean, safe Anchor `0.29.0` compliant code.
   - Guard against integer overflow and rounding inaccuracies. Always round calculations in favor of the protocol reserve.
2. **Commit Conventions**:
   Follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat(...)`: New features
   - `fix(...)`: Bug fixes
   - `docs(...)`: Documentation updates
   - `infra(...)`: CI/CD or Docker configurations
3. **Pull Request Workflow**:
   - Run `python cli.py demo` locally before submitting PRs.
   - Ensure zero warnings under `flake8`.
