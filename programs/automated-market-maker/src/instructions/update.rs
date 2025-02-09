use crate::{error::AMMError, state::*};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateArgs {
    pub locked: Option<bool>,
    pub fee: Option<u16>,
    pub authority: Option<Pubkey>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority @ AMMError::InvalidConfigAuthority,
    )]
    pub config: Account<'info, Config>,
}

impl<'info> Update<'info> {
    pub fn update(&mut self, args: UpdateArgs) -> Result<()> {
        if let Some(locked) = args.locked {
            self.config.locked = locked;
        }

        if let Some(fee) = args.fee {
            self.config.fee = fee;
        }

        if let Some(authority) = args.authority {
            self.config.authority = authority;
        }

        Ok(())
    }
}
