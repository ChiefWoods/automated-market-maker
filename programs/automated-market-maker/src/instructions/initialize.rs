use crate::{constants::*, state::*};
use anchor_lang::{prelude::*, Discriminator};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeArgs {
    pub seed: u64,
    pub locked: bool,
    pub fee: u16,
}

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        seeds = [CONFIG_SEED, args.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub config: AccountLoader<'info, Config>,
    #[account(
        init,
        payer = authority,
        seeds = [LP_SEED, config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,
    #[account(mint::token_program = token_program)]
    pub mint_x: Box<InterfaceAccount<'info, Mint>>,
    #[account(mint::token_program = token_program)]
    pub mint_y: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: InitializeBumps, args: InitializeArgs) -> Result<()> {
        let config = &mut self.config.load_init()?;

        // config.set_inner(Config {
        //     seed: args.seed,
        //     locked: args.locked,
        //     bump: bumps.config,
        //     lp_bump: bumps.mint_lp,
        //     fee: args.fee,
        //     mint_x: self.mint_x.key(),
        //     mint_y: self.mint_y.key(),
        //     authority: self.authority.key(),
        // });

        config.seed = args.seed;
        // config.locked = args.locked;
        config.bump = bumps.config;
        config.lp_bump = bumps.mint_lp;
        config.fee = args.fee;
        config.mint_x = self.mint_x.key();
        config.mint_y = self.mint_y.key();
        config.authority = self.authority.key();

        Ok(())
    }
}
