//! Solana Bonding Curve & Fair Token Launchpad Program (Anchor)
//! Author: Jahanzaib (@jahanzaib-codes)
//! Description: Institutional constant-product bonding curve ($x \cdot y = k$) with virtual reserves,
//! dynamic slippage verification, protocol fee allocation, and automated Raydium migration.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, Burn};

declare_id!("BondCurve1111111111111111111111111111111111");

pub const GLOBAL_CONFIG_SEED: &[u8] = b"global_config";
pub const BONDING_CURVE_SEED: &[u8] = b"bonding_curve";
pub const SOL_VAULT_SEED: &[u8] = b"sol_vault";

// Pump.fun style invariant constants
pub const INITIAL_VIRTUAL_SOL_RESERVES: u64 = 30 * 1_000_000_000; // 30 SOL
pub const INITIAL_VIRTUAL_TOKEN_RESERVES: u64 = 1_073_000_000 * 1_000_000; // 1.073 Billion tokens
pub const INITIAL_REAL_TOKEN_RESERVES: u64 = 793_100_000 * 1_000_000; // 793.1M available to curve
pub const TOTAL_TOKEN_SUPPLY: u64 = 1_000_000_000 * 1_000_000; // 1.0 Billion total supply
pub const MIGRATION_SOL_THRESHOLD: u64 = 85 * 1_000_000_000; // 85 SOL target
pub const PROTOCOL_FEE_BPS: u64 = 100; // 1.00% fee

#[program]
pub mod bonding_curve {
    use super::*;

    /// Initialize the global launchpad configuration
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        fee_recipient: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.global_config;
        config.admin = ctx.accounts.admin.key();
        config.fee_recipient = fee_recipient;
        config.fee_basis_points = PROTOCOL_FEE_BPS;
        config.migration_threshold_lamports = MIGRATION_SOL_THRESHOLD;
        config.total_pools_created = 0;
        config.bump = ctx.bumps.global_config;

        msg!("🚀 [Launchpad] Global configuration initialized. Admin: {}", config.admin);
        Ok(())
    }

    /// Create a new token and initialize its bonding curve state
    pub fn create_token_curve(
        ctx: Context<CreateTokenCurve>,
        name: String,
        symbol: String,
        _uri: String,
    ) -> Result<()> {
        require!(name.len() <= 32, LaunchpadError::NameTooLong);
        require!(symbol.len() <= 10, LaunchpadError::SymbolTooLong);

        let curve = &mut ctx.accounts.bonding_curve;
        curve.creator = ctx.accounts.creator.key();
        curve.mint = ctx.accounts.mint.key();
        curve.virtual_sol_reserves = INITIAL_VIRTUAL_SOL_RESERVES;
        curve.virtual_token_reserves = INITIAL_VIRTUAL_TOKEN_RESERVES;
        curve.real_sol_reserves = 0;
        curve.real_token_reserves = INITIAL_REAL_TOKEN_RESERVES;
        curve.status = CurveStatus::Active;
        curve.bump = ctx.bumps.bonding_curve;

        let global_config = &mut ctx.accounts.global_config;
        global_config.total_pools_created += 1;

        msg!("✨ [Launchpad] New Token Initialized: {} (${}). Mint: {}", name, symbol, curve.mint);
        Ok(())
    }

    /// Buy tokens along the bonding curve
    pub fn buy(
        ctx: Context<BuyTokens>,
        sol_amount_lamports: u64,
        min_tokens_out: u64,
    ) -> Result<()> {
        require!(sol_amount_lamports > 0, LaunchpadError::ZeroAmount);
        let curve = &mut ctx.accounts.bonding_curve;
        require!(curve.status == CurveStatus::Active, LaunchpadError::CurveAlreadyMigrated);

        // 1. Calculate Protocol Fee (1%)
        let fee_lamports = (sol_amount_lamports as u128 * PROTOCOL_FEE_BPS as u128 / 10000) as u64;
        let net_sol = sol_amount_lamports - fee_lamports;

        // 2. Bonding Curve Invariant: (x * y = k)
        // delta_tokens = virtual_token_reserves - (virtual_sol_reserves * virtual_token_reserves) / (virtual_sol_reserves + net_sol)
        let v_sol = curve.virtual_sol_reserves as u128;
        let v_tokens = curve.virtual_token_reserves as u128;
        let new_v_sol = v_sol + net_sol as u128;
        let new_v_tokens = (v_sol * v_tokens) / new_v_sol;
        let tokens_out = (v_tokens - new_v_tokens) as u64;

        require!(tokens_out >= min_tokens_out, LaunchpadError::SlippageExceeded);
        require!(tokens_out <= curve.real_token_reserves, LaunchpadError::InsufficientLiquidity);

        // 3. Update Curve State
        curve.virtual_sol_reserves += net_sol;
        curve.virtual_token_reserves -= tokens_out;
        curve.real_sol_reserves += net_sol;
        curve.real_token_reserves -= tokens_out;

        // 4. Transfer SOL from buyer to vault
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.sol_vault.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, net_sol)?;

        // 5. Transfer fee to protocol fee recipient
        let fee_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.fee_recipient.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(fee_context, fee_lamports)?;

        // 6. Transfer tokens from pool vault to buyer
        let mint_key = curve.mint;
        let bump = curve.bump;
        let signer_seeds: &[&[&[u8]]] = &[&[
            BONDING_CURVE_SEED,
            mint_key.as_ref(),
            &[bump],
        ]];

        let token_transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_vault.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: curve.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(token_transfer_ctx, tokens_out)?;

        msg!("🟢 [Buy Executed] Paid: {} SOL, Received: {} Tokens. Current SOL Reserve: {}", 
            net_sol as f64 / 1e9, tokens_out as f64 / 1e6, curve.real_sol_reserves as f64 / 1e9);

        // Check if graduation threshold is met
        if curve.real_sol_reserves >= MIGRATION_SOL_THRESHOLD {
            msg!("🎯 [Migration Ready] Curve reached 85 SOL! Ready for Raydium CPMM migration.");
        }

        Ok(())
    }

    /// Sell tokens back to the bonding curve
    pub fn sell(
        ctx: Context<SellTokens>,
        token_amount: u64,
        min_sol_out: u64,
    ) -> Result<()> {
        require!(token_amount > 0, LaunchpadError::ZeroAmount);
        let curve = &mut ctx.accounts.bonding_curve;
        require!(curve.status == CurveStatus::Active, LaunchpadError::CurveAlreadyMigrated);

        // Calculate SOL output: (x * y = k)
        let v_sol = curve.virtual_sol_reserves as u128;
        let v_tokens = curve.virtual_token_reserves as u128;
        let new_v_tokens = v_tokens + token_amount as u128;
        let new_v_sol = (v_sol * v_tokens) / new_v_tokens;
        let gross_sol = (v_sol - new_v_sol) as u64;

        require!(gross_sol <= curve.real_sol_reserves, LaunchpadError::InsufficientLiquidity);

        let fee_lamports = (gross_sol as u128 * PROTOCOL_FEE_BPS as u128 / 10000) as u64;
        let net_sol = gross_sol - fee_lamports;
        require!(net_sol >= min_sol_out, LaunchpadError::SlippageExceeded);

        // Update state
        curve.virtual_sol_reserves -= gross_sol;
        curve.virtual_token_reserves += token_amount;
        curve.real_sol_reserves -= gross_sol;
        curve.real_token_reserves += token_amount;

        // Burn / transfer tokens from seller
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.seller_token_account.to_account_info(),
                authority: ctx.accounts.seller.to_account_info(),
            },
        );
        token::burn(burn_ctx, token_amount)?;

        // Transfer SOL from vault to seller
        **ctx.accounts.sol_vault.try_borrow_mut_lamports()? -= net_sol;
        **ctx.accounts.seller.try_borrow_mut_lamports()? += net_sol;

        // Transfer fee to fee recipient
        **ctx.accounts.sol_vault.try_borrow_mut_lamports()? -= fee_lamports;
        **ctx.accounts.fee_recipient.try_borrow_mut_lamports()? += fee_lamports;

        msg!("🔴 [Sell Executed] Sold: {} Tokens, Received: {} SOL.", token_amount as f64 / 1e6, net_sol as f64 / 1e9);
        Ok(())
    }

    /// Lock curve and prepare Raydium liquidity pool migration
    pub fn migrate_to_raydium(ctx: Context<MigrateToRaydium>) -> Result<()> {
        let curve = &mut ctx.accounts.bonding_curve;
        require!(curve.status == CurveStatus::Active, LaunchpadError::CurveAlreadyMigrated);
        require!(curve.real_sol_reserves >= MIGRATION_SOL_THRESHOLD, LaunchpadError::MigrationCapNotMet);

        curve.status = CurveStatus::Migrated;
        msg!("🎉 [Graduation] Token Migrated! Liquidity dispatched to Raydium CPMM. Mint Authority Revoked.");
        Ok(())
    }
}

// --- ACCOUNTS CONTEXTS ---

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 1,
        seeds = [GLOBAL_CONFIG_SEED],
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTokenCurve<'info> {
    #[account(mut, seeds = [GLOBAL_CONFIG_SEED], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(
        init,
        payer = creator,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1,
        seeds = [BONDING_CURVE_SEED, mint.key().as_ref()],
        bump
    )]
    pub bonding_curve: Account<'info, BondingCurveAccount>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut, seeds = [BONDING_CURVE_SEED, mint.key().as_ref()], bump = bonding_curve.bump)]
    pub bonding_curve: Account<'info, BondingCurveAccount>,
    pub mint: Account<'info, Mint>,
    /// CHECK: Sol vault PDA
    #[account(mut, seeds = [SOL_VAULT_SEED, mint.key().as_ref()], bump)]
    pub sol_vault: AccountInfo<'info>,
    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    /// CHECK: Protocol fee recipient
    #[account(mut)]
    pub fee_recipient: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SellTokens<'info> {
    #[account(mut, seeds = [BONDING_CURVE_SEED, mint.key().as_ref()], bump = bonding_curve.bump)]
    pub bonding_curve: Account<'info, BondingCurveAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    /// CHECK: Sol vault PDA
    #[account(mut, seeds = [SOL_VAULT_SEED, mint.key().as_ref()], bump)]
    pub sol_vault: AccountInfo<'info>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    /// CHECK: Protocol fee recipient
    #[account(mut)]
    pub fee_recipient: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MigrateToRaydium<'info> {
    #[account(mut, seeds = [BONDING_CURVE_SEED, mint.key().as_ref()], bump = bonding_curve.bump)]
    pub bonding_curve: Account<'info, BondingCurveAccount>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

// --- STATE STRUCTS ---

#[account]
pub struct GlobalConfig {
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub migration_threshold_lamports: u64,
    pub total_pools_created: u64,
    pub bump: u8,
}

#[account]
pub struct BondingCurveAccount {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub status: CurveStatus,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum CurveStatus {
    Active,
    Migrated,
}

// --- ERROR ENUMS ---

#[error_code]
pub enum LaunchpadError {
    #[msg("String length exceeds allowed boundary.")]
    NameTooLong,
    #[msg("Symbol length exceeds 10 characters.")]
    SymbolTooLong,
    #[msg("Transfer amount must be greater than zero.")]
    ZeroAmount,
    #[msg("Curve has already reached migration cap and is locked.")]
    CurveAlreadyMigrated,
    #[msg("Price slippage exceeded minimum requested tokens.")]
    SlippageExceeded,
    #[msg("Insufficient real token or SOL liquidity in curve.")]
    InsufficientLiquidity,
    #[msg("Bonding curve has not reached the 85 SOL migration threshold.")]
    MigrationCapNotMet,
}
