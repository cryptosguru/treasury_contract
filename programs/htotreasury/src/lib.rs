use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::mem::size_of;


declare_id!("GfXYYi5TFPG5ixdfiXLQacgjZbatqpb9uZZPMTBxMVCx");

const FULL_100: u64 = 100_000_000_000;
const ACC_PRECISION: u128 = 100_000_000_000;

#[program]
pub mod hto_treasury {
    use super::*;

    pub fn create_state(
        _ctx: Context<CreateState>,
        bump: u8,
        token_per_second: u64,
    ) -> ProgramResult {
        let state = &mut _ctx.accounts.state.load_init()?;
        state.authority = _ctx.accounts.authority.key();
        state.bump = bump;
        state.start_time = _ctx.accounts.clock.unix_timestamp;
        state.token_per_second = token_per_second;
        state.reward_mint = _ctx.accounts.reward_mint.key();
        state.reward_vault = _ctx.accounts.reward_vault.key();
        Ok(())
    }


    pub fn change_tokens_per_second(
        _ctx: Context<ChangeTokensPerSecond>,
        token_per_second: u64,
    ) -> ProgramResult {
        let mut state = _ctx.accounts.state.load_mut()?;
        let provided_remaining_accounts = &mut _ctx.remaining_accounts.iter();
        let mut i = 0;
        
        while i < _ctx.remaining_accounts.len(){
            i +=1;
            let provided_token_accountinfo = next_account_info(provided_remaining_accounts)?;
            let loader = Loader::<GoldPoolAccount>::try_from(&_ctx.program_id, &provided_token_accountinfo)?;
            loader.load_mut()?.update(&state, &_ctx.accounts.clock)?;
        }
        state.token_per_second = token_per_second;
        emit!(RateChanged { token_per_second });
        Ok(())
    }

    pub fn create_pool(
        _ctx: Context<CreateGoldPool>,
        bump: u8,
        
    ) -> ProgramResult {
         let mut state = _ctx.accounts.state.load_mut()?;

        let pool = &mut _ctx.accounts.pool.load_init()?;
        pool.bump = bump;
        pool.mint = _ctx.accounts.mint.key();
        pool.vault = _ctx.accounts.vault.key();
        
        pool.amount_multipler = amount_multipler;
        pool.authority = _ctx.accounts.authority.key();

        

        emit!(PoolCreated {
            pool: _ctx.accounts.pool.key(),
            mint: _ctx.accounts.mint.key()
        });
        Ok(())
    }

    pub fn close_pool(_ctx: Context<CloseGoldPool>) -> ProgramResult {
        let mut state = _ctx.accounts.state.load_mut()?;

        let provided_remaining_accounts = &mut _ctx.remaining_accounts.iter();
        let mut i = 0;
        
        while i < _ctx.remaining_accounts.len(){
            i +=1;

            let provided_token_accountinfo = next_account_info(provided_remaining_accounts)?;
            let loader = Loader::<GoldPoolAccount>::try_from(&_ctx.program_id, &provided_token_accountinfo)?;
            loader.load_mut()?.update(&state, &_ctx.accounts.clock)?;
        }


        let pool = _ctx.accounts.pool.load()?;
        require!(pool.amount == 0, ErrorCode::WorkingPool);
      
        Ok(())
    }

    pub fn change_pool_amount_multipler(
        _ctx: Context<ChangePoolSetting>,
        amount_multipler: u64,
    ) -> ProgramResult {
        let mut pool = _ctx.accounts.pool.load_mut()?;
        pool.amount_multipler = amount_multipler;
        emit!(PoolAmountMultiplerChanged {
            pool: _ctx.accounts.pool.key(),
            amount_multipler
        });
        Ok(())
    }

  
    pub fn create_user(_ctx: Context<CreatePoolUser>, bump: u8) -> ProgramResult {
        let user = &mut _ctx.accounts.user.load_init()?;
        user.authority = _ctx.accounts.authority.key();
        user.bump = bump;
        user.pool = _ctx.accounts.pool.key();

        let mut pool = _ctx.accounts.pool.load_mut()?;
        pool.total_user += 1;
        emit!(UserCreated {
            pool: _ctx.accounts.pool.key(),
            user: _ctx.accounts.user.key(),
            authority: _ctx.accounts.authority.key(),
        });
        Ok(())
    }

    pub fn create_user_sol_address(
        _ctx: Context<CreateUserSolAddress>,
        bump: u8,
        sol_address: String,
    ) -> ProgramResult {
        let user = &mut _ctx.accounts.user.load_init()?;
        let src = sol_address.as_bytes();
        let mut data = [0u8; 42];
        data[..src.len()].copy_from_slice(src);
        user.sol_address = data;
        user.bump = bump;
        user.authority = _ctx.accounts.authority.key();
        emit!(UserSolAddressChanged {
            authority: _ctx.accounts.authority.key(),
            sol_address
        });
        Ok(())
    }

    pub fn set_user_sol_address(
        _ctx: Context<SetUserSolAddress>,
        sol_address: String,
    ) -> ProgramResult {
        let mut user = _ctx.accounts.user.load_mut()?;
        let src = sol_address.as_bytes();
        let mut data = [0u8; 42];
        data[..src.len()].copy_from_slice(src);
        user.sol_address = data;
        emit!(UserSolAddressChanged {
            authority: _ctx.accounts.authority.key(),
            sol_address
        });
        Ok(())
    }

    pub fn select(_ctx: Context<select>, amount: u64) -> ProgramResult {
        msg!("treasuring...");
        let state = _ctx.accounts.state.load()?;

        let mut user = _ctx.accounts.user.load_mut()?;
        let mut pool = _ctx.accounts.pool.load_mut()?;
        msg!("loaded states");

        
        msg!("updated state");
        
        user.calculate_reward_amount(&pool)?;
        msg!("calculate_reward_amount");
        user.amount = user.amount.checked_add(amount).unwrap();
        pool.amount = pool.amount.checked_add(amount).unwrap();

     
        user.last_select_time = _ctx.accounts.clock.unix_timestamp;

        let cpi_accounts = Transfer {
            from: _ctx.accounts.user_vault.to_account_info(),
            to: _ctx.accounts.pool_vault.to_account_info(),
            authority: _ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = _ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        msg!("selectd {}", amount);
        emit!(UserSelected {
            pool: _ctx.accounts.pool.key(),
            user: _ctx.accounts.user.key(),
            authority: _ctx.accounts.authority.key(),
            amount
        });
        Ok(())
    }

    pub fn claim(_ctx: Context<select>, amount: u64) -> ProgramResult {
        
        let state = _ctx.accounts.state.load()?;
        let mut user = _ctx.accounts.user.load_mut()?;
        let mut pool = _ctx.accounts.pool.load_mut()?;

        require!(user.amount >= amount, ErrorCode::ClaimOverAmount);
        require!(
            user.last_select_time
                <= _ctx.accounts.clock.unix_timestamp,
            ErrorCode::UnderLocked
        );

        user.last_select_time = _ctx.accounts.clock.unix_timestamp;
        user.amount = user.amount.checked_sub(amount).unwrap();
        pool.amount = pool.amount.checked_sub(amount).unwrap();

       

        //user.calculate_reward_debt(&pool)?;
        drop(pool);

        let new_pool = _ctx.accounts.pool.load()?;
        let cpi_accounts = Transfer {
            from: _ctx.accounts.pool_vault.to_account_info(),
            to: _ctx.accounts.user_vault.to_account_info(),
            authority: _ctx.accounts.pool.to_account_info(),
        };

        let seeds = &[new_pool.mint.as_ref(), &[new_pool.bump]];
        let signer = &[&seeds[..]];
        let cpi_program = _ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;
        emit!(UserClaimed {
            pool: _ctx.accounts.pool.key(),
            user: _ctx.accounts.user.key(),
            authority: _ctx.accounts.authority.key(),
            amount
        });
        Ok(())
    }

    pub fn harvest(_ctx: Context<Harvest>) -> ProgramResult {
        
        let state = _ctx.accounts.state.load()?;
        let mut pool = _ctx.accounts.pool.load_mut()?;
        let mut user = _ctx.accounts.user.load_mut()?;

        user.calculate_reward_amount(&pool)?;

        let total_reward= u64(user.reward_amount);

        let cpi_accounts = Transfer {
            from: _ctx.accounts.reward_vault.to_account_info(),
            to: _ctx.accounts.user_vault.to_account_info(),
            authority: _ctx.accounts.state.to_account_info(),
        };

        let seeds = &[b"state".as_ref(), &[state.bump]];
        let signer = &[&seeds[..]];
        let cpi_program = _ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, total_reward)?;

    //    user.reward_amount = 0;
    
        emit!(UserHarvested {
            pool: _ctx.accounts.pool.key(),
            user: _ctx.accounts.user.key(),
            authority: _ctx.accounts.authority.key(),
            amount: total_reward
        });
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateState<'info> {
    #[account(
        init,
        seeds = [b"state".as_ref()],
        bump = bump,
        payer = authority,
        space = 8 + size_of::<StateAccount>()
    )]
    pub state: Loader<'info, StateAccount>,
    #[account(constraint = reward_vault.owner == state.key())]
    pub reward_vault: Account<'info, TokenAccount>,
    pub reward_mint: Box<Account<'info, Mint>>,
    pub authority: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}


#[derive(Accounts)]
pub struct ChangeTokensPerSecond<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump = state.load()?.bump, has_one = authority)]
    pub state: Loader<'info, StateAccount>,
    pub authority: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateGoldPool<'info> {
    #[account(
        init,
        seeds = [mint.key().as_ref()],
        bump = bump,
        payer = authority,
        space = 8 + size_of::<GoldPoolAccount>()
    )]
    pub pool: Loader<'info, GoldPoolAccount>,
    #[account(mut, seeds = [b"state".as_ref()], bump = state.load()?.bump, has_one = authority)]
    pub state: Loader<'info, StateAccount>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(constraint = vault.owner == pool.key())]
    pub vault: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct CloseGoldPool<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump = state.load()?.bump, has_one = authority)]
    pub state: Loader<'info, StateAccount>,
    #[account(mut, seeds = [pool.load()?.mint.key().as_ref()], bump = pool.load()?.bump, has_one = authority, close = authority)]
    pub pool: Loader<'info, GoldPoolAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ChangePoolSetting<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump = state.load()?.bump)]
    pub state: Loader<'info, StateAccount>,
    #[account(mut, seeds = [pool.load()?.mint.key().as_ref()], bump = pool.load()?.bump, has_one = authority)]
    pub pool: Loader<'info, GoldPoolAccount>,
    pub authority: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}


#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreatePoolUser<'info> {
    #[account(
        init,
        seeds = [pool.key().as_ref(), authority.key().as_ref()],
        bump = bump,
        payer = authority,
        space = 8 + size_of::<GoldPoolUserAccount>()
    )]
    pub user: Loader<'info, GoldPoolUserAccount>,
    #[account(seeds = [b"state".as_ref()], bump = state.load()?.bump)]
    pub state: Loader<'info, StateAccount>,
    #[account(mut, seeds = [pool.load()?.mint.key().as_ref()], bump = pool.load()?.bump)]
    pub pool: Loader<'info, GoldPoolAccount>,
    pub authority: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateUserSolAddress<'info> {
    #[account(
        init,
        seeds = [b"sol".as_ref(), authority.key().as_ref()],
        bump = bump,
        payer = authority,
        space = 8 + size_of::<GoldUserSolAddress>()
    )]
    pub user: Loader<'info, GoldUserSolAddress>,
    pub authority: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SetUserSolAddress<'info> {
    #[account(mut, seeds = [b"sol".as_ref(), authority.key().as_ref()], bump = user.load()?.bump, has_one = authority)]
    pub user: Loader<'info, GoldUserSolAddress>,
    pub authority: Signer<'info>,
    pub system_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct select<'info> {
    #[account(mut, seeds = [pool.key().as_ref(), authority.key().as_ref()], bump = user.load()?.bump, has_one = pool, has_one = authority)]
    pub user: Loader<'info, GoldPoolUserAccount>,
    #[account(mut, seeds = [b"state".as_ref()], bump = state.load()?.bump)]
    pub state: Loader<'info, StateAccount>,
    
    #[account(mut, seeds = [pool.load()?.mint.key().as_ref()], bump = pool.load()?.bump)]
    pub pool: Loader<'info, GoldPoolAccount>,
    pub authority: Signer<'info>,
    #[account(constraint = mint.key() == pool.load()?.mint)]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut, constraint = pool_vault.owner == pool.key())]
    pub pool_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_vault.owner == authority.key())]
    pub user_vault: Box<Account<'info, TokenAccount>>,
    pub system_program: UncheckedAccount<'info>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct Harvest<'info> {
    #[account(mut, seeds = [pool.key().as_ref(), authority.key().as_ref()], bump = user.load()?.bump, has_one = pool, has_one = authority)]
    pub user: Loader<'info, GoldPoolUserAccount>,
    #[account(mut, seeds = [b"state".as_ref()], bump = state.load()?.bump)]
    pub state: Loader<'info, StateAccount>,
    #[account(mut, seeds = [pool.load()?.mint.key().as_ref()], bump = pool.load()?.bump)]
    pub pool: Loader<'info, GoldPoolAccount>,
    pub authority: Signer<'info>,
    #[account(constraint = mint.key() == pool.load()?.mint)]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut, constraint = reward_vault.owner == state.key())]
    pub reward_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_vault.owner == authority.key())]
    pub user_vault: Box<Account<'info, TokenAccount>>,
    pub system_program: UncheckedAccount<'info>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[account(zero_copy)]
pub struct StateAccount {
    pub authority: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_vault: Pubkey,
    pub bump: u8,

    pub start_time: i64,
    pub token_per_second: u64,
}

#[account(zero_copy,has_one = authority)]
pub struct GoldPoolAccount {
    pub bump: u8,
    pub authority: Pubkey,
    pub amount: u64,
    pub mint: Pubkey,
    pub vault: Pubkey,
    
    pub total_user: u64,
}

impl GoldPoolAccount {
    
}



#[account(zero_copy)]
pub struct GoldPoolUserAccount {
    pub bump: u8,
    pub pool: Pubkey,
    pub authority: Pubkey,
    pub amount: u64,
    pub reward_amount: u128,
    pub last_select_time: i64,
    pub reserved_1: u128,
    pub reserved_2: u128,
    pub reserved_3: u128,
}

#[account(zero_copy)]
pub struct GoldUserSolAddress {
    pub bump: u8,
    pub authority: Pubkey,
    pub sol_address: [u8; 42],
}

impl GoldPoolUserAccount {
    fn calculate_reward_amount<'info>(
        &mut self,
        pool: &GoldPoolAccount,
    ) -> Result<()> {
        let pending_amount: u128 = u128::from(self.amount);
        self.reward_amount = self.reward_amount.checked_add(pending_amount).unwrap();
       
        Ok(())
    }
   
}

#[error]
pub enum ErrorCode {
    #[msg("Over selectd amount")]
    ClaimOverAmount,
    #[msg("Under locked")]
    UnderLocked,
    #[msg("Pool is working")]
    WorkingPool,
    #[msg("Invalid Lock Duration")]
    InvalidLockDuration,
    #[msg("Invalid SEQ")]
    InvalidSEQ,
}
#[event]
pub struct RateChanged {
    token_per_second: u64,
}
#[event]
pub struct PoolCreated {
    pool: Pubkey,
    mint: Pubkey,
}
#[event]
pub struct PoolLockDurationChanged {
    pool: Pubkey,
   
}
#[event]
pub struct PoolAmountMultiplerChanged {
    pool: Pubkey,
    amount_multipler: u64,
}

#[event]
pub struct UserCreated {
    pool: Pubkey,
    user: Pubkey,
    authority: Pubkey,
}
#[event]
pub struct UserSolAddressChanged {
    authority: Pubkey,
    sol_address: String,
}
#[event]
pub struct UserSelected {
    pool: Pubkey,
    user: Pubkey,
    authority: Pubkey,
    amount: u64,   
}
#[event]
pub struct UserClaimed {
    pool: Pubkey,
    user: Pubkey,
    authority: Pubkey,
    amount: u64,
}
#[event]
pub struct UserHarvested {
    pool: Pubkey,
    user: Pubkey,
    authority: Pubkey,
    amount: u64,
}
