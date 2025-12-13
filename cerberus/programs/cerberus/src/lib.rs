use anchor_lang::prelude::*;

declare_id!("HopC35nDjfRjRGYEvao9y3j3EN2iqtqKJ6Zkj6MaeshD");

#[program]
pub mod cerberus {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
