use crate::{constants::*, error::AMMError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SwapArgs {
    is_x: bool,
    amount: u64,
    min: u64,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        // has_one = mint_x,
        // has_one = mint_y,
        // seeds = [CONFIG_SEED, config.seed.to_le_bytes().as_ref()],
        // bump = config.bump,
    )]
    pub config: AccountLoader<'info, Config>,
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

impl<'info> Swap<'info> {
    pub fn swap(&self, args: SwapArgs) -> Result<()> {
        // self.config.invariant()?;
        require_gt!(args.amount, 0, AMMError::InvalidAmount);

        let config = self.config.load()?;

        let mut curve = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.vault_x.amount,
            config.fee,
            None,
        )
        .unwrap();

        let p = match args.is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y,
        };

        let res = curve.swap(p, args.amount, args.min).unwrap();

        require_neq!(res.deposit, 0, AMMError::InvalidAmount);
        require_neq!(res.withdraw, 0, AMMError::InvalidAmount);

        let (from, to, mint, decimals) = match args.is_x {
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
            res.deposit,
            decimals,
        )?;

        let (from, to, mint, decimals) = match args.is_x {
            true => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
            false => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
        };

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
            res.withdraw,
            decimals,
        )
    }
}
