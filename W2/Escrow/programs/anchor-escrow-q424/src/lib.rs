pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("DgUNYfGZE5giS2oJCtspXPnpwJ1mhp6WS6ceKv1abk5k");

#[program]
pub mod anchor_escrow_q424 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
