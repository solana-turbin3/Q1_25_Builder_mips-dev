use anchor_lang::prelude::*;

declare_id!("E4X2D9QnjbBsZMYXyG1ZGzXabCM5y3AWKgQ2TzRFibRG");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
