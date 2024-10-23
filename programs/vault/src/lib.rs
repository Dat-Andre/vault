use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

declare_id!("8NSk348tJdgP66cZrkEZcVkVz5Krd9TYzakABe948v9a");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Payment>) -> Result<()> {
        ctx.accounts.close_vault()
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [b"state".as_ref(), user.key().as_ref()],
        bump 
    )]
    pub state: Account<'info, VaultState>,
    #[account(
        seeds = [b"vault".as_ref(), state.key().as_ref()],
        bump 
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.state.vault_bump = bumps.vault;
        self.state.state_bumb = bumps.state;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Payment<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"state".as_ref(), user.key().as_ref()],
        bump= state.state_bumb
    )]
    pub state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"vault".as_ref(), state.key().as_ref()],
        bump= state.vault_bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Payment<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer{
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        let _ = transfer(cpi_context, amount);
       
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let seeds = &[b"vault", self.state.to_account_info().key.as_ref(), &[self.state.vault_bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer{
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        let _ = transfer(cpi_context, amount);
        Ok(())
    } 

    pub fn close_vault(&mut self) -> Result<()> {

        let cpi_program = self.system_program.to_account_info();

        let seeds = &[b"vault", self.state.to_account_info().key.as_ref(), &[self.state.vault_bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer{
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        let _ = transfer(cpi_context, self.vault.lamports());
        Ok(())
    }

    pub fn close_state(&mut self) -> Result<()> {

        let cpi_program = self.system_program.to_account_info();

        let seeds = &[b"state", self.user.to_account_info().key.as_ref(), &[self.state.state_bumb]];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer{
            from: self.state.to_account_info(),
            to: self.user.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        let _ = transfer(cpi_context, self.vault.lamports());
        Ok(())
    }
    
}

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bumb: u8,
}

/* 
Not needed anymore, InitSpace is implemented in the Anchor framework
impl Space for VaultState {
    const INIT_SPACE: usize = 8 + 1 + 1;
} 
*/
