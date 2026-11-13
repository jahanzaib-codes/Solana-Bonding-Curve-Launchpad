"""Mathematical engine for constant-product Solana bonding curves."""

from ..models.curve_models import TokenCurveState, TradeExecution


class BondingCurveMathEngine:
    def __init__(self, sol_price_usd: float = 215.0):
        self.sol_price_usd = sol_price_usd
        self.fee_bps = 100  # 1.00% protocol fee

    def update_metrics(self, state: TokenCurveState) -> None:
        state.current_price_sol = state.virtual_sol_reserves / state.virtual_token_reserves
        state.current_market_cap_usd = (
            state.current_price_sol * self.sol_price_usd * state.total_token_supply
        )
        if state.real_sol_reserves >= state.migration_threshold_sol:
            state.is_migrated = True

    def calculate_buy(
        self,
        state: TokenCurveState,
        trade_id: str,
        trader: str,
        sol_in: float
    ) -> TradeExecution:
        fee_sol = sol_in * (self.fee_bps / 10000.0)
        net_sol = sol_in - fee_sol

        k = state.virtual_sol_reserves * state.virtual_token_reserves
        new_v_sol = state.virtual_sol_reserves + net_sol
        new_v_tokens = k / new_v_sol
        tokens_out = state.virtual_token_reserves - new_v_tokens

        tokens_out = min(tokens_out, state.real_token_reserves)

        # Update curve
        state.virtual_sol_reserves = new_v_sol
        state.virtual_token_reserves = new_v_tokens
        state.real_sol_reserves += net_sol
        state.real_token_reserves -= tokens_out

        self.update_metrics(state)

        progress = min(100.0, (state.real_sol_reserves / state.migration_threshold_sol) * 100.0)
        unit_price = (net_sol / tokens_out) if tokens_out > 0 else state.current_price_sol

        return TradeExecution(
            trade_id=trade_id,
            trader_wallet=trader,
            trade_type="BUY",
            sol_amount=sol_in,
            token_amount=tokens_out,
            unit_price_sol=unit_price,
            protocol_fee_sol=fee_sol,
            resulting_real_sol=state.real_sol_reserves,
            resulting_market_cap_usd=state.current_market_cap_usd,
            progress_to_graduation_pct=progress
        )

    def calculate_sell(
        self,
        state: TokenCurveState,
        trade_id: str,
        trader: str,
        tokens_in: float
    ) -> TradeExecution:
        k = state.virtual_sol_reserves * state.virtual_token_reserves
        new_v_tokens = state.virtual_token_reserves + tokens_in
        new_v_sol = k / new_v_tokens
        gross_sol = state.virtual_sol_reserves - new_v_sol

        gross_sol = min(gross_sol, state.real_sol_reserves)
        fee_sol = gross_sol * (self.fee_bps / 10000.0)
        net_sol = gross_sol - fee_sol

        # Update curve
        state.virtual_sol_reserves = new_v_sol
        state.virtual_token_reserves = new_v_tokens
        state.real_sol_reserves -= gross_sol
        state.real_token_reserves += tokens_in

        self.update_metrics(state)

        progress = min(100.0, (state.real_sol_reserves / state.migration_threshold_sol) * 100.0)
        unit_price = (gross_sol / tokens_in) if tokens_in > 0 else state.current_price_sol

        return TradeExecution(
            trade_id=trade_id,
            trader_wallet=trader,
            trade_type="SELL",
            sol_amount=net_sol,
            token_amount=tokens_in,
            unit_price_sol=unit_price,
            protocol_fee_sol=fee_sol,
            resulting_real_sol=state.real_sol_reserves,
            resulting_market_cap_usd=state.current_market_cap_usd,
            progress_to_graduation_pct=progress
        )
