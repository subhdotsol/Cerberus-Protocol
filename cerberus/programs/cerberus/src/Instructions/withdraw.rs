use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::states::*;
use crate::events::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // Step 1: Distributor must exist and signer must be authority
    #[account(
        seeds = [b"distributor"],
        bump = distributor.bump,
        has_one = authority @ CerberusError::Unauthorized
    )]
    pub distributor: Account<'info, MerkleDistributor>,
    
    // Step 2: Vault must be mutable for withdrawal
    /// CHECK: Vault is validated against distributor.vault constraint
    #[account(
        mut,
        constraint = vault.key() == distributor.vault @ CerberusError::VaultMismatch
    )]
    pub vault: UncheckedAccount<'info>,
    
    // Step 3: Recipient token account
    /// CHECK: Recipient is validated by token program during transfer
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,
    
    // Step 4: Authority must sign
    pub authority: Signer<'info>,
    
    // Step 5: Token program for CPI
    pub token_program: Program<'info, Token>,
}

pub fn withdraw(
    ctx: Context<Withdraw>,
    amount: u64,
) -> Result<()> {
    // Step 1: Get reference to distributor
    let distributor = &ctx.accounts.distributor;
    
    // Step 2: Verify signer is the authority
    // This is enforced by the `has_one = authority` constraint
    
    // Step 3: Verify vault matches distributor's vault
    require!(
        ctx.accounts.vault.key() == distributor.vault,
        CerberusError::VaultMismatch
    );
    
    // Step 4: Token transfer will fail if vault has insufficient balance
    // The SPL token program will handle this validation
    
    // Step 5: Prepare PDA signer seeds
    let seeds = &[
        b"distributor".as_ref(),
        &[distributor.bump],
    ];
    let signer = &[&seeds[..]];
    
    // Step 6: Create CPI context for token transfer
    let cpi_accounts = Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.recipient.to_account_info(),
        authority: distributor.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    // Step 7: Execute the token transfer
    token::transfer(cpi_ctx, amount)?;
    
    // Step 8: Get current timestamp
    let clock = Clock::get()?;
    
    // Step 9: Emit withdrawal event
    emit!(Withdrawn {
        distributor: distributor.key(),
        authority: ctx.accounts.authority.key(),
        recipient: ctx.accounts.recipient.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });
    
    // Step 10: Log withdrawal details
    msg!(
        "Withdrawn {} tokens to {}",
        amount,
        ctx.accounts.recipient.key()
    );
    
    Ok(())
}
