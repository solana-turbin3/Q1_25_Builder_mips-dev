use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("FJdwdjJTM5xfEno2H38uPPu5mMebyFbNSGhif6ekcEjy"); // Replace with your actual program ID

#[program]
pub mod escrow_bidding {
    use super::*;

    // Initializes an escrow account for the user (bidder) and deposits an initial amount if provided.
    pub fn init_escrow(ctx: Context<InitEscrow>, initial_deposit: u64) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        escrow.owner = ctx.accounts.owner.key();
        escrow.deposited_amount = 0;
        escrow.locked_amount = 0;
        escrow.bump = *ctx.bumps.get("escrow").unwrap();

        // If an initial deposit is provided, transfer SOL from the owner to the escrow PDA.
        if initial_deposit > 0 {
            let escrow_info = escrow.to_account_info();
            let owner_info = ctx.accounts.owner.to_account_info();
            let cpi_ctx = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: owner_info,
                    to: escrow_info,
                },
            );
            transfer(cpi_ctx, initial_deposit)?;
            escrow.deposited_amount = initial_deposit;
        }
        Ok(())
    }

    // Deposit additional SOL into an existing escrow account.
    pub fn deposit_funds(ctx: Context<DepositFunds>, deposit_amount: u64) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let escrow_info = escrow.to_account_info();
        let owner_info = ctx.accounts.owner.to_account_info();
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: owner_info,
                to: escrow_info,
            },
        );
        transfer(cpi_ctx, deposit_amount)?;
        escrow.deposited_amount = escrow.deposited_amount.checked_add(deposit_amount).unwrap();
        Ok(())
    }

    // Place a bid by locking the specified SOL amount from the escrow.
    pub fn place_bid(ctx: Context<PlaceBid>, bid_amount: u64) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        // Ensure that available funds (deposited minus locked) cover the bid.
        let available = escrow.deposited_amount.checked_sub(escrow.locked_amount).unwrap();
        require!(available >= bid_amount, CustomError::InsufficientFunds);

        // Initialize the new bid account.
        let bid = &mut ctx.accounts.bid;
        bid.escrow = escrow.key();
        bid.bidder = ctx.accounts.bidder.key();
        bid.amount = bid_amount;
        bid.active = true;

        // Increase the locked amount in escrow.
        escrow.locked_amount = escrow.locked_amount.checked_add(bid_amount).unwrap();
        Ok(())
    }

    // Cancel an active bid. The bid’s locked funds are immediately refunded
    // (transferred from the escrow PDA back to the bidder) and the bid is marked inactive.
    pub fn cancel_bid(ctx: Context<CancelBid>) -> Result<()> {
        // Get the bid amount and ensure the bid is active.
        let bid = &mut ctx.accounts.bid;
        require!(bid.active, CustomError::BidNotActive);
        let bid_amount = bid.amount;
    
        // Manually adjust lamports: refund bid_amount from escrow PDA to bidder.
        let escrow_info = ctx.accounts.escrow.to_account_info();
        let bidder_info = ctx.accounts.bidder.to_account_info();
    
        **escrow_info.lamports.borrow_mut() = escrow_info
            .lamports()
            .checked_sub(bid_amount)
            .ok_or(CustomError::InsufficientFunds)?;
        **bidder_info.lamports.borrow_mut() = bidder_info
            .lamports()
            .checked_add(bid_amount)
            .ok_or(CustomError::InsufficientFunds)?;
    
        // Update the escrow state.
        let escrow = &mut ctx.accounts.escrow;
        escrow.locked_amount = escrow.locked_amount.checked_sub(bid_amount).unwrap();
        escrow.deposited_amount = escrow.deposited_amount.checked_sub(bid_amount).unwrap();
    
        // Mark the bid as inactive.
        bid.active = false;
        Ok(())
    }    

    // Resolve (finalize) a winning bid. The winning bid’s funds are permanently deducted
    // from the escrow (i.e. considered spent) and its state is updated to inactive.
    pub fn resolve_bid(ctx: Context<ResolveBid>) -> Result<()> {
        let bid = &mut ctx.accounts.winning_bid;
        require!(bid.active, CustomError::BidNotActive);
        bid.active = false;

        let escrow = &mut ctx.accounts.escrow;
        // Deduct the winning bid amount from both deposited and locked funds.
        escrow.deposited_amount = escrow.deposited_amount.checked_sub(bid.amount).unwrap();
        escrow.locked_amount = escrow.locked_amount.checked_sub(bid.amount).unwrap();

        Ok(())
    }

  // Withdraw available (unlocked) funds from the escrow back to the owner’s wallet.
  pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amount: u64) -> Result<()> {
    // Capture current state from escrow.
    let current_deposited = ctx.accounts.escrow.deposited_amount;
    let current_locked = ctx.accounts.escrow.locked_amount;
    let available = current_deposited.checked_sub(current_locked).unwrap();
    require!(available >= amount, CustomError::InsufficientFunds);

    {
        // Adjust lamport balances in an inner block.
        let escrow_info = ctx.accounts.escrow.to_account_info();
        let owner_info = ctx.accounts.owner.to_account_info();

        **escrow_info.lamports.borrow_mut() = escrow_info
            .lamports()
            .checked_sub(amount)
            .ok_or(CustomError::InsufficientFunds)?;
        **owner_info.lamports.borrow_mut() = owner_info
            .lamports()
            .checked_add(amount)
            .ok_or(CustomError::InsufficientFunds)?;
    }

    {
        // Now update the stored deposited_amount using a fresh mutable borrow.
        let escrow = &mut ctx.accounts.escrow;
        escrow.deposited_amount = current_deposited.checked_sub(amount).unwrap();
    }
    Ok(())
}
}

#[derive(Accounts)]
pub struct InitEscrow<'info> {
    #[account(
        init,
        seeds = [b"escrow", owner.key().as_ref()],
        bump,
        payer = owner,
        space = 8 + Escrow::LEN + 16
    )]
    pub escrow: Account<'info, Escrow>,    
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositFunds<'info> {
    #[account(mut, seeds = [b"escrow", owner.key().as_ref()], bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut, seeds = [b"escrow", escrow.owner.as_ref()], bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(
        init,
        payer = bidder,
        space = 8 + Bid::LEN
    )]
    pub bid: Account<'info, Bid>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelBid<'info> {
    #[account(mut, seeds = [b"escrow", escrow.owner.as_ref()], bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(mut, constraint = bid.escrow == escrow.key(), constraint = bid.bidder == bidder.key())]
    pub bid: Account<'info, Bid>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResolveBid<'info> {
    #[account(mut, seeds = [b"escrow", escrow.owner.as_ref()], bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(mut, constraint = winning_bid.escrow == escrow.key())]
    pub winning_bid: Account<'info, Bid>,
    #[account(mut, address = escrow.owner)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut, seeds = [b"escrow", owner.key().as_ref()], bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Escrow {
    pub owner: Pubkey,
    pub deposited_amount: u64, // Total SOL held in escrow.
    pub locked_amount: u64,    // Funds locked in active bids.
    pub bump: u8,              // PDA bump seed.
}

impl Escrow {
    pub const LEN: usize = 32 + 8 + 8 + 1; // owner + deposited_amount + locked_amount + bump
}

#[account]
pub struct Bid {
    pub escrow: Pubkey,  // Associated escrow account.
    pub bidder: Pubkey,  // Should match escrow.owner.
    pub amount: u64,     // Amount locked for this bid.
    pub active: bool,    // True if the bid is still active.
}

impl Bid {
    pub const LEN: usize = 32 + 32 + 8 + 1; // escrow + bidder + amount + active
}

#[error_code]
pub enum CustomError {
    #[msg("Insufficient funds available in escrow.")]
    InsufficientFunds,
    #[msg("Bid is not active.")]
    BidNotActive,
}