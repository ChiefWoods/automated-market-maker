use crate::{constants::*, state::*};
use anchor_lang::{prelude::*, Discriminator};
use anchor_spl::token_interface::{Mint, TokenInterface};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeConfigArgs {
    pub seed: u64,
    pub locked: bool,
    pub fee: u16,
}

#[derive(Accounts)]
#[instruction(args: InitializeConfigArgs)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        seeds = [CONFIG_SEED, args.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = authority,
        seeds = [LP_SEED, config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
        mint::token_program = token_program,
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,
    #[account(mint::token_program = token_program)]
    pub mint_x: InterfaceAccount<'info, Mint>,
    #[account(mint::token_program = token_program)]
    pub mint_y: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl InitializeConfig<'_> {
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        args: InitializeConfigArgs,
    ) -> Result<()> {
        ctx.accounts.config.set_inner(Config {
            seed: args.seed,
            locked: args.locked,
            bump: ctx.bumps.config,
            lp_bump: ctx.bumps.mint_lp,
            fee: args.fee,
            mint_x: ctx.accounts.mint_x.key(),
            mint_y: ctx.accounts.mint_y.key(),
            authority: ctx.accounts.authority.key(),
        });

        Ok(())
    }
}
