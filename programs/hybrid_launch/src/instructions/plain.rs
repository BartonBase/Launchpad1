//! Mode 1 native path — `launch_plain` (Barton 2026-10-01).
//!
//! Creates a classic SPL mint, mints exactly 1B, revokes mint authority, never sets freeze, and
//! locks a [`PlainLaunchConfig`] at `["launch_config", mint]`. The supply sits in the ATA of the
//! `launch_vault` PDA. No instruction signs with that PDA. This path is not a curve. The curve
//! path is `register_plain_dbc`. There is no wrap, unwrap, NFT, collection, ratio, or fee transfer.

use anchor_lang::{
    prelude::*,
    system_program::{self, Allocate, Assign, CreateAccount, Transfer},
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken},
    token::{
        self, spl_token::instruction::AuthorityType, InitializeMint2, Mint, MintTo, SetAuthority,
        Token, TokenAccount,
    },
};

use crate::{
    constants::*,
    error::LaunchError,
    state::PlainLaunchConfig,
    validation::{plain_total_supply, PlainLaunchParams},
};

#[derive(Accounts)]
pub struct LaunchPlain<'info> {
    /// Pays rent. Recorded as `creator`; holds no powers.
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Fresh keypair. Must sign. System-owned and empty (pre-funded lamports are topped up).
    /// An existing classic or Token-2022 mint cannot be onboarded.
    #[account(mut)]
    pub mint: Signer<'info>,

    /// Immutable Mode 1 record. One per mint; the same seeds as a hybrid config, so the modes cannot stack.
    #[account(
        init,
        payer = creator,
        space = 8 + PlainLaunchConfig::INIT_SPACE,
        seeds = [LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        bump
    )]
    pub launch_config: Account<'info, PlainLaunchConfig>,

    /// CHECK: data-less PDA; mint authority for the one `mint_to`, then revoked.
    #[account(seeds = [MINT_AUTHORITY_SEED, launch_config.key().as_ref()], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: data-less PDA that owns the destination ATA. Not caller-chosen. Nothing signs with these seeds.
    #[account(seeds = [LAUNCH_VAULT_SEED, mint.key().as_ref(), launch_config.key().as_ref()], bump)]
    pub launch_vault: UncheckedAccount<'info>,

    /// CHECK: must equal ATA(launch_vault, mint, classic Token); created by CPI.
    #[account(mut)]
    pub launch_destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_launch_plain(ctx: Context<LaunchPlain>, params: PlainLaunchParams) -> Result<()> {
    let total_supply_base = plain_total_supply(params.decimals)?;
    let token_program_id = ctx.accounts.token_program.key();
    let mint_key = ctx.accounts.mint.key();
    let config_key = ctx.accounts.launch_config.key();

    require_keys_eq!(
        ctx.accounts.launch_destination.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.launch_vault.key(),
            &mint_key,
            &token_program_id
        ),
        LaunchError::InvalidLaunchDestination
    );

    let space = <anchor_spl::token::spl_token::state::Mint as anchor_lang::solana_program::program_pack::Pack>::LEN;
    create_plain_mint_account(&ctx, space, &token_program_id)?;

    token::initialize_mint2(
        CpiContext::new(
            token_program_id,
            InitializeMint2 {
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        params.decimals,
        &ctx.accounts.mint_authority.key(),
        None,
    )?;

    associated_token::create(CpiContext::new(
        ctx.accounts.associated_token_program.key(),
        associated_token::Create {
            payer: ctx.accounts.creator.to_account_info(),
            associated_token: ctx.accounts.launch_destination.to_account_info(),
            authority: ctx.accounts.launch_vault.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    ))?;

    let bump = ctx.bumps.mint_authority;
    let seeds: &[&[u8]] = &[MINT_AUTHORITY_SEED, config_key.as_ref(), &[bump]];
    let signer = &[seeds];
    token::mint_to(
        CpiContext::new_with_signer(
            token_program_id,
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.launch_destination.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer,
        ),
        total_supply_base,
    )?;
    token::set_authority(
        CpiContext::new_with_signer(
            token_program_id,
            SetAuthority {
                current_authority: ctx.accounts.mint_authority.to_account_info(),
                account_or_mint: ctx.accounts.mint.to_account_info(),
            },
            signer,
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    {
        let info = ctx.accounts.mint.to_account_info();
        require_keys_eq!(
            *info.owner,
            SPL_TOKEN_PROGRAM_ID,
            LaunchError::PostLaunchCheckFailed
        );
        let data = info.try_borrow_data()?;
        let mint = Mint::try_deserialize(&mut &data[..])?;
        require!(
            mint.supply == total_supply_base
                && mint.decimals == params.decimals
                && mint.mint_authority.is_none()
                && mint.freeze_authority.is_none(),
            LaunchError::PostLaunchCheckFailed
        );
        let dest = ctx.accounts.launch_destination.to_account_info();
        require_keys_eq!(
            *dest.owner,
            SPL_TOKEN_PROGRAM_ID,
            LaunchError::PostLaunchCheckFailed
        );
        let d = TokenAccount::try_deserialize(&mut &dest.try_borrow_data()?[..])?;
        require!(
            d.owner == ctx.accounts.launch_vault.key()
                && d.mint == mint_key
                && d.amount == total_supply_base
                && d.delegate.is_none()
                && d.close_authority.is_none(),
            LaunchError::PostLaunchCheckFailed
        );
    }

    ctx.accounts.launch_config.set_inner(PlainLaunchConfig {
        version: PLAIN_LAUNCH_CONFIG_VERSION,
        bump: ctx.bumps.launch_config,
        mint_authority_bump: bump,
        launch_mode: LAUNCH_MODE_PLAIN,
        creator: ctx.accounts.creator.key(),
        mint: mint_key,
        launch_destination: ctx.accounts.launch_destination.key(),
        launch_vault: ctx.accounts.launch_vault.key(),
        launch_vault_bump: ctx.bumps.launch_vault,
        decimals: params.decimals,
        total_supply_base,
        launched_at: Clock::get()?.unix_timestamp,
        dbc_config: Pubkey::default(),
        dbc_pool: Pubkey::default(),
        graduation_threshold_lamports: 0,
    });
    msg!(
        "hybrid_launch: plain mint {} supply {}",
        mint_key,
        total_supply_base
    );
    Ok(())
}

/// Same pre-fund handling as Mode 2 `launch` (QA-HL-01). A system-owned, data-less account that
/// already holds lamports is topped up, allocated, and assigned. Anything else is rejected.
fn create_plain_mint_account(
    ctx: &Context<LaunchPlain>,
    space: usize,
    token_program_id: &Pubkey,
) -> Result<()> {
    let sys = ctx.accounts.system_program.key();
    let mint = ctx.accounts.mint.to_account_info();
    let creator = ctx.accounts.creator.to_account_info();
    let rent_min = Rent::get()?.minimum_balance(space);
    let current = mint.lamports();
    if current == 0 {
        return system_program::create_account(
            CpiContext::new(
                sys,
                CreateAccount {
                    from: creator,
                    to: mint,
                },
            ),
            rent_min,
            space as u64,
            token_program_id,
        );
    }
    require_keys_eq!(
        *mint.owner,
        system_program::ID,
        LaunchError::MintAccountInUse
    );
    require!(
        mint.data_is_empty() && !mint.executable,
        LaunchError::MintAccountInUse
    );
    let top_up = rent_min.saturating_sub(current);
    if top_up > 0 {
        system_program::transfer(
            CpiContext::new(
                sys,
                Transfer {
                    from: creator,
                    to: mint.clone(),
                },
            ),
            top_up,
        )?;
    }
    system_program::allocate(
        CpiContext::new(
            sys,
            Allocate {
                account_to_allocate: mint.clone(),
            },
        ),
        space as u64,
    )?;
    system_program::assign(
        CpiContext::new(
            sys,
            Assign {
                account_to_assign: mint,
            },
        ),
        token_program_id,
    )?;
    Ok(())
}
