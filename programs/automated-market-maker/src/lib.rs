pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("C7f2vQaRdp8oJR5R9P86CKn48wQkEmgtdkEpjZy1MaAi");

#[program]
pub mod automated_market_maker {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        args: InitializeConfigArgs,
    ) -> Result<()> {
        ctx.accounts.initialize_config(ctx.bumps, args)
    }

    pub fn initialize_vaults(ctx: Context<InitializeVaults>) -> Result<()> {
        ctx.accounts.initialize_vaults()
    }

    pub fn update(ctx: Context<Update>, args: UpdateArgs) -> Result<()> {
        ctx.accounts.update(args)
    }

    pub fn deposit(ctx: Context<Deposit>, args: DepositArgs) -> Result<()> {
        ctx.accounts.deposit(args)
    }

    pub fn withdraw(ctx: Context<Withdraw>, args: WithdrawArgs) -> Result<()> {
        ctx.accounts.withdraw(args)
    }

    pub fn swap(ctx: Context<Swap>, args: SwapArgs) -> Result<()> {
        ctx.accounts.swap(args)
    }
}
