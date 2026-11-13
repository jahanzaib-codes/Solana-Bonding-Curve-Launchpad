# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-11-13

### Added
- **Anchor Rust Smart Contract**:
  - `lib.rs`: Constant-product bonding curve ($x \cdot y = k$) with 30 SOL virtual reserves and 1.073B virtual token supply.
  - Mathematical buying and selling curves with 1.00% protocol fee deduction.
  - Automatic 85 SOL graduation cap triggering curve freeze and Raydium CPMM migration.
- **Python Mathematical Engine & Interactive CLI**:
  - `math_engine.py`: Precise integer and float invariant emulation for slippage and pricing.
  - `cli.py`: Interactive Click CLI with ASCII tables tracing trade inflows, market cap growth, and graduation.
- **Infrastructure & Containerization**:
  - Multi-stage Dockerfile and Docker Compose orchestration.
  - Dual-version (Python 3.10 & 3.11) matrix CI/CD via GitHub Actions.
