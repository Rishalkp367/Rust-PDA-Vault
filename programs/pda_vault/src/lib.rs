use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("6xDR8chaRseMHDBRFEGLAT2na9GCX5Bubc5Wowzev1Qx");

#[program]
pub mod pda_vault {
    use super::*;

    // -------------------------
    // Initialize the global vault
    // -------------------------
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault_state = &mut ctx.accounts.vault_state;

        vault_state.admin = ctx.accounts.admin.key();
        vault_state.total_deposited = 0;
        vault_state.state_bump = ctx.bumps.vault_state;
        vault_state.vault_bump = ctx.bumps.vault;

        Ok(())
    }

    // -------------------------
    // Initialize per-user state
    // -------------------------
    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        let user_state = &mut ctx.accounts.user_state;

        user_state.user = ctx.accounts.user.key();
        user_state.deposited = 0;
        user_state.bump = ctx.bumps.user_state;

        Ok(())
    }

    // -------------------------
    // Deposit SOL
    // -------------------------
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        let ix = system_program::Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        };

        system_program::transfer(
            CpiContext::new(ctx.accounts.system_program.to_account_info(), ix),
            amount,
        )?;

        ctx.accounts.user_state.deposited =
            ctx.accounts.user_state.deposited.checked_add(amount).ok_or(VaultError::MathOverflow)?;

        ctx.accounts.vault_state.total_deposited =
            ctx.accounts.vault_state.total_deposited.checked_add(amount).ok_or(VaultError::MathOverflow)?;

        Ok(())
    }

    // -------------------------
    // Withdraw SOL
    // -------------------------
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);
        require!(ctx.accounts.user_state.deposited >= amount, VaultError::InsufficientDepositedFunds);

        let vault_balance = ctx.accounts.vault.to_account_info().lamports();
        require!(vault_balance >= amount, VaultError::InsufficientVaultBalance);

        let seeds = &[
            b"vault".as_ref(),
            &[ctx.accounts.vault_state.vault_bump],
        ];
        let signer = &[&seeds[..]];

        let ix = system_program::Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
        };

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                ix,
                signer,
            ),
            amount,
        )?;

        ctx.accounts.user_state.deposited -= amount;
        ctx.accounts.vault_state.total_deposited -= amount;

        Ok(())
    }
}

/* =========================
        ACCOUNTS
========================= */

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [b"vault_state"],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    /// CHECK: PDA vault holding SOL only
    #[account(
        init,
        payer = admin,
        space = 0,
        owner = system_program::ID,
        seeds = [b"vault"],
        bump
    )]
    pub vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + UserState::INIT_SPACE,
        seeds = [b"user_state", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [b"vault_state"],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    /// CHECK: vault PDA
    #[account(
        mut,
        seeds = [b"vault"],
        bump = vault_state.vault_bump
    )]
    pub vault: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"user_state", user.key().as_ref()],
        bump = user_state.bump,
        has_one = user
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"vault_state"],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    /// CHECK: vault PDA
    #[account(
        mut,
        seeds = [b"vault"],
        bump = vault_state.vault_bump
    )]
    pub vault: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"user_state", user.key().as_ref()],
        bump = user_state.bump,
        has_one = user
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/* =========================
          STATE
========================= */

#[account]
pub struct VaultState {
    pub admin: Pubkey,
    pub total_deposited: u64,
    pub state_bump: u8,
    pub vault_bump: u8,
}

impl VaultState {
    pub const INIT_SPACE: usize = 32 + 8 + 1 + 1;
}

#[account]
pub struct UserState {
    pub user: Pubkey,
    pub deposited: u64,
    pub bump: u8,
}

impl UserState {
    pub const INIT_SPACE: usize = 32 + 8 + 1;
}

/* =========================
          ERRORS
========================= */

#[error_code]
pub enum VaultError {
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Insufficient deposited funds")]
    InsufficientDepositedFunds,
    #[msg("Vault has insufficient lamports")]
    InsufficientVaultBalance,
    #[msg("Math overflow")]
    MathOverflow,
}
