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
    pub config: AccountLoader<'info, Config>,
}

impl<'info> Update<'info> {
    pub fn update(&mut self, args: UpdateArgs) -> Result<()> {
        let config = &mut self.config.load_mut()?;

        // if let Some(locked) = args.locked {
        //     config.locked = locked;
        // }

        if let Some(fee) = args.fee {
            config.fee = fee;
        }

        if let Some(authority) = args.authority {
            config.authority = authority;
        }

        Ok(())
    }
}
