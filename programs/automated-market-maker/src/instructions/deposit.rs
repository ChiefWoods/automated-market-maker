use crate::{constants::*, error::AMMError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};
use constant_product_curve::ConstantProduct;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositArgs {
    amount: u64,
    max_x: u64,
    max_y: u64,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint, decimals) = match is_x {
            true => (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    authority: self.user.to_account_info(),
                    from,
                    to,
                    mint,
                },
            ),
            amount,
            decimals,
        )
    }

    pub fn deposit(&self, args: DepositArgs) -> Result<()> {
        // self.config.invariant()?;
        require_gt!(args.amount, 0, AMMError::InvalidAmount);

        let (amount_x, amount_y) = match self.mint_lp.supply == 0
            && self.vault_x.amount == 0
            && self.vault_y.amount == 0
        {
            true => (args.max_x, args.max_y),
            false => {
                let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    args.amount,
                    6,
                )
                .unwrap();

                (amounts.x, amounts.y)
            }
        };

        require!(
            amount_x <= args.max_x && amount_y <= args.max_y,
            AMMError::SlippageExceeded
        );

        self.deposit_tokens(true, amount_x)?;
        self.deposit_tokens(false, amount_y)?;

        let config = self.config.load()?;

        let signer_seeds: &[&[&[u8]]] =
            &[&[CONFIG_SEED, &config.seed.to_le_bytes(), &[config.bump]]];

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    authority: self.config.to_account_info(),
                    mint: self.mint_lp.to_account_info(),
                    to: self.user_lp.to_account_info(),
                },
                signer_seeds,
            ),
            args.amount,
        )
    }
}
