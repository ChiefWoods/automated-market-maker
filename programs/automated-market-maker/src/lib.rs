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

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
