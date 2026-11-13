"""Data models for Solana bonding curve mechanics and trade telemetry."""

from pydantic import BaseModel, Field
from typing import List


class TokenCurveState(BaseModel):
    name: str
    symbol: str
    mint: str
    virtual_sol_reserves: float = 30.0
    virtual_token_reserves: float = 1_073_000_000.0
    real_sol_reserves: float = 0.0
    real_token_reserves: float = 793_100_000.0
    total_token_supply: float = 1_000_000_000.0
    current_price_sol: float = Field(default=0.0)
    current_market_cap_usd: float = Field(default=0.0)
    migration_threshold_sol: float = 85.0
    is_migrated: bool = False


class TradeExecution(BaseModel):
    trade_id: str
    trader_wallet: str
    trade_type: str  # "BUY" | "SELL"
    sol_amount: float
    token_amount: float
    unit_price_sol: float
    protocol_fee_sol: float
    resulting_real_sol: float
    resulting_market_cap_usd: float
    progress_to_graduation_pct: float


class CurveSimulationSummary(BaseModel):
    token: TokenCurveState
    trades: List[TradeExecution]
    final_sol_reserve: float
    graduated_to_raydium: bool
    total_protocol_fees_sol: float
