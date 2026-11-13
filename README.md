# Solana Bonding Curve Launchpad 🪐⚡

[![CI](https://github.com/jahanzaib-codes/Solana-Bonding-Curve-Launchpad/actions/workflows/ci.yml/badge.svg)](https://github.com/jahanzaib-codes/Solana-Bonding-Curve-Launchpad/actions/workflows/ci.yml)
[![Anchor Rust](https://img.shields.io/badge/Anchor-0.29.0-orange.svg)](https://anchor-lang.com/)
[![Solana](https://img.shields.io/badge/Solana-Mainnet%20%7C%20Devnet-9945FF.svg)](https://solana.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Docker Ready](https://img.shields.io/badge/docker-ready-brightgreen.svg)](Dockerfile)

A production-grade **Solana Token Launchpad and Constant-Product Bonding Curve Program** engineered in **Anchor Rust**. Implements mathematical price discovery, virtual reserve initialization, protocol fee distribution, dynamic slippage protection, and automated liquidity graduation to **Raydium CPMM**.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        SOLANA BONDING CURVE ARCHITECTURE                               │
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│   [ Token Genesis / Deployer ] ──► [ Bonding Curve PDA Program ]                       │
│                                           │                                            │
│                                           ├─ Virtual SOL: 30 SOL                       │
│                                           ├─ Virtual Token: 1.073B                     │
│                                           └─ Real Available Tokens: 793.1M             │
│                                                   │                                    │
│   [ Traders / Snipers ] ──(SOL Buy/Sell)──────────┼──(1% Protocol Fee)──► [ Treasury ] │
│                                                   │                                    │
│                                                   ▼ (Constant-Product: x * y = k)      │
│   [ 85 SOL Graduation Threshold Reached ] ────────┘                                    │
│             │                                                                          │
│             ▼                                                                          │
│   [ Curve Locked & Mint Authority Revoked ] ──► [ Raydium CPMM Liquidity Ingestion ]   │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 🌟 Key Features

- **Constant-Product Mathematical Invariant ($x \cdot y = k$)**:
  Initializes virtual reserves ($30\text{ SOL}$ and $1.073\text{B Tokens}$) ensuring progressive, smooth price discovery from genesis micro-caps up to graduation without sudden liquidity cliffs.
- **Dynamic Slippage & Minimum Output Guarantees**:
  Enforces on-chain `min_tokens_out` and `min_sol_out` checks to protect traders against frontrunning and sandwich attacks.
- **Non-Custodial Fee Splitting**:
  Deducts a configurable protocol fee (default: $1.00\%$) directly to the fee recipient PDA on every buy and sell execution.
- **Automated Raydium CPMM Graduation**:
  Once the curve accumulates $85\text{ SOL}$ in real reserves, trading locks automatically, token mint authority is permanently revoked, and funds migrate to initialize a Raydium CPMM liquidity pool.

---

## 📐 Mathematical Formulation

### 1. Buy Calculation (Tokens Out for $\Delta \text{SOL}_{\text{net}}$)
$$\Delta \text{Tokens} = V_{\text{tokens}} - \frac{V_{\text{sol}} \cdot V_{\text{tokens}}}{V_{\text{sol}} + \Delta \text{SOL}_{\text{net}}}$$
Where $\Delta \text{SOL}_{\text{net}} = \Delta \text{SOL}_{\text{in}} \cdot (1 - \text{Fee})$.

### 2. Sell Calculation ($\text{SOL Out}$ for $\Delta \text{Tokens}$)
$$\Delta \text{SOL}_{\text{gross}} = V_{\text{sol}} - \frac{V_{\text{sol}} \cdot V_{\text{tokens}}}{V_{\text{tokens}} + \Delta \text{Tokens}_{\text{in}}}$$
$$\Delta \text{SOL}_{\text{net}} = \Delta \text{SOL}_{\text{gross}} \cdot (1 - \text{Fee})$$

---

## 🚀 Quick Start & CLI Demo

### 1. Installation

```bash
git clone https://github.com/jahanzaib-codes/Solana-Bonding-Curve-Launchpad.git
cd Solana-Bonding-Curve-Launchpad
pip install -r requirements.txt
```

### 2. Run Interactive Terminal Simulation

```bash
# Run full Genesis-to-Graduation simulation
python cli.py demo
```

### Terminal Output Preview

```
=========================================================================================================
   🪐 SOLANA-BONDING-CURVE-LAUNCHPAD: CONSTANT-PRODUCT ANCHOR PROGRAM DEMO
=========================================================================================================

[1/3] Token Genesis & Virtual Reserve Invariant:
  • Token:                  Quantum Solana AI ($QSAI)
  • Mint Address:           7pB8Q9gRtY4kP9jK1mZ8vL3wQ2eR4tY6uI8oP0aSDFG
  • Virtual SOL Reserve:    30.00 SOL (Pump.fun mathematical invariant)
  • Graduation Threshold:   85.00 SOL ($18,275 USD) -> Auto-Migrate to Raydium CPMM

[2/3] Live Bonding Curve Trade Execution Stream:
| Tx ID      | Trader                 | Action   | Inflow       | Output   | Market Cap (USD)   | Pool SOL   | Graduation %   |
|------------|------------------------|----------|--------------|----------|--------------------|------------|----------------|
| TX-BUY-01  | 7xKXtg...AlphaWhale    | 🟢 BUY   | 5.00 SOL     | 151.97M  | $8,158.53          | 4.95 SOL   | 5.8%           |
| TX-BUY-02  | 4ND1mB...InsiderSniper | 🟢 BUY   | 12.00 SOL    | 233.65M  | $14,647.58         | 16.83 SOL  | 19.8%          |
| TX-BUY-03  | 9WzDXw...ArbEngine     | 🟢 BUY   | 25.00 SOL    | 237.67M  | $34,221.64         | 41.58 SOL  | 48.9%          |
| TX-SELL-04 | 4ND1mB...InsiderSniper | 🔴 SELL  | 40.0M Tokens | 5.79 SOL | $28,859.41         | 35.73 SOL  | 42.0%          |
| TX-BUY-06  | 3mK9L2...EcosystemFund | 🟢 BUY   | 22.00 SOL    | 57.40M   | $91,763.67         | 87.21 SOL  | 100.0%         |

[3/3] Raydium CPMM Graduation & Migration Status:
  🎉 STATUS: GRADUATION THRESHOLD ACHIEVED (>= 85.0 SOL)
     • Final Real SOL Reserve: 87.21 SOL (~$18,750.85 USD)
     • Protocol Fee Accrued:   0.9985 SOL
     • Raydium CPMM Ingestion: Bonding curve frozen; liquidity dispatched to Raydium pool.
     • SPL Mint Authority:     REVOKED (Immutable token supply guaranteed).
```

---

## 🐳 Docker Deployment

```bash
docker-compose up -d --build
docker-compose logs -f bonding-curve-engine
```

---

## 📁 Repository Structure

```
Solana-Bonding-Curve-Launchpad/
├── programs/
│   └── bonding-curve/
│       ├── Cargo.toml                # Anchor Rust program dependencies
│       └── src/
│           └── lib.rs                # Core Anchor bonding curve program logic
├── src/
│   ├── curve/
│   │   └── math_engine.py            # High-precision constant-product math engine
│   ├── models/
│   │   └── curve_models.py           # Pydantic state & trade models
│   └── simulator/
│       └── launchpad_simulator.py    # Multi-scenario fair-launch simulation harness
├── Anchor.toml                       # Anchor configuration & network profile
├── Cargo.toml                        # Workspace configuration
├── cli.py                            # Interactive Click terminal interface
├── main.py                           # Standalone entrypoint
├── Dockerfile                        # Multi-stage production container
├── docker-compose.yml                # Microservice container orchestration
├── requirements.txt                  # Python dependencies
├── CONTRIBUTING.md                   # Open source contribution guidelines
├── SECURITY.md                       # Security policy
├── CHANGELOG.md                      # Release notes history
└── README.md                         # Architecture documentation
```

---

## 🤝 Professional Consultation & Custom Launchpad Development

Looking for custom Solana Anchor smart contracts, bonding curve launchpads, Raydium CPMM integration, or token snipers?

- **Fiverr**: [jahanzaib313110](https://www.fiverr.com/jahanzaib313110)
- **Telegram**: [@JanzebBlockdev](https://t.me/JanzebBlockdev)

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).
