"""Simulation runner demonstrating fair launch bonding curve mechanics."""

from ..models.curve_models import TokenCurveState, CurveSimulationSummary
from ..curve.math_engine import BondingCurveMathEngine


def run_bonding_curve_simulation() -> CurveSimulationSummary:
    engine = BondingCurveMathEngine(sol_price_usd=215.0)

    token = TokenCurveState(
        name="Quantum Solana AI",
        symbol="QSAI",
        mint="7pB8Q9gRtY4kP9jK1mZ8vL3wQ2eR4tY6uI8oP0aSDFG",
        virtual_sol_reserves=30.0,
        virtual_token_reserves=1_073_000_000.0,
        real_sol_reserves=0.0,
        real_token_reserves=793_100_000.0,
        total_token_supply=1_000_000_000.0,
        migration_threshold_sol=85.0
    )
    engine.update_metrics(token)

    trades = [
        # 1. Dev / Genesis Seed buy
        ("TX-BUY-01", "7xKXtg...AlphaWhale", "BUY", 5.0),
        # 2. Fast Sniper Inflow
        ("TX-BUY-02", "4ND1mB...InsiderSniper", "BUY", 12.0),
        # 3. Community Momentum Inflow
        ("TX-BUY-03", "9WzDXw...ArbEngine", "BUY", 25.0),
        # 4. Partial Profit Taking
        ("TX-SELL-04", "4ND1mB...InsiderSniper", "SELL", 40_000_000.0),
        # 5. Whale Surge Buying
        ("TX-BUY-05", "7xKXtg...AlphaWhale", "BUY", 30.0),
        # 6. Final Push to 85 SOL Graduation Threshold
        ("TX-BUY-06", "3mK9L2...EcosystemFund", "BUY", 22.0)
    ]

    trade_results = []
    total_fees = 0.0

    for tx_id, trader, t_type, amount in trades:
        if t_type == "BUY":
            res = engine.calculate_buy(token, tx_id, trader, amount)
        else:
            res = engine.calculate_sell(token, tx_id, trader, amount)

        total_fees += res.protocol_fee_sol
        trade_results.append(res)

    return CurveSimulationSummary(
        token=token,
        trades=trade_results,
        final_sol_reserve=token.real_sol_reserves,
        graduated_to_raydium=token.is_migrated,
        total_protocol_fees_sol=total_fees
    )
