use crate::{constants::*, error::AMMError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};
use constant_product_curve::{ConstantProduct, XYAmounts};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawArgs {
    amount: u64,
    min_x: u64,
    min_y: u64,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        // has_one = mint_x,
        // has_one = mint_y,
        // seeds = [CONFIG_SEED, config.seed.to_le_bytes().as_ref()],
        // bump = config.bump,
    )]
    pub config: AccountLoader<'info, Config>,
    #[account(
        mut,
        // seeds = [LP_SEED, config.key().as_ref()],
        // bump = config.lp_bump,
    )]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,
    #[account(mint::token_program = token_program)]
    pub mint_x: Box<InterfaceAccount<'info, Mint>>,
    #[account(mint::token_program = token_program)]
    pub mint_y: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint, decimals) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        let config = self.config.load()?;

        let signer_seeds: &[&[&[u8]]] =
            &[&[CONFIG_SEED, &config.seed.to_le_bytes(), &[config.bump]]];

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    authority: self.config.to_account_info(),
                    from,
                    to,
                    mint,
                },
                signer_seeds,
            ),
            amount,
            decimals,
        )
    }

    pub fn withdraw(&self, args: WithdrawArgs) -> Result<()> {
        // self.config.invariant()?;
        require_gt!(args.amount, 0, AMMError::InvalidAmount);
        require!(
            args.min_x != 0 && args.min_y != 0,
            AMMError::InvalidMinAmount
        );

        let XYAmounts {
            x: amount_x,
            y: amount_y,
        } = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            args.amount,
            6,
        )
        .unwrap();

        require!(
            args.min_x <= amount_x && args.min_y <= amount_y,
            AMMError::SlippageExceeded
        );

        self.withdraw_tokens(true, amount_x)?;
        self.withdraw_tokens(false, amount_y)?;

        burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    authority: self.user.to_account_info(),
                    from: self.user_lp.to_account_info(),
                    mint: self.mint_lp.to_account_info(),
                },
            ),
            args.amount,
        )
    }
}
