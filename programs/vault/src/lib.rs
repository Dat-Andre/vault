use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

declare_id!("8NSk348tJdgP66cZrkEZcVkVz5Krd9TYzakABe948v9a");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"state".as_ref(), user.key().as_ref()],
        bump 
    )]
    pub state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"vaults".as_ref(), user.key().as_ref()],
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
        seeds = [b"vault".as_ref(), user.key().as_ref()],
        bump= state.state_bumb
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        seeds = [b"state".as_ref(), user.key().as_ref()],
        bump= state.vault_bump
    )]
    pub state: Account<'info, VaultState>,
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