//! `launch`: create a Track A classic SPL Token mint with a fixed 1B supply and
//! an immutable LaunchConfig, in ONE instruction:
//! 1. validate params (ratio set, 100 <= collection_size, collection_size * ratio <= 1B via
//!    checked_mul, collection_size <= MAX_COLLECTION_SIZE; the flat SOL fee is looked up from the
//!    ratio tier table, never supplied; there is no token fee, ADR-013);
//! 2. create the 82-byte mint account owned by the CLASSIC Token program
//!    (`token_program: Program<Token>` rejects Token-2022: INV-13). Pre-funded
//!    but empty system accounts are tolerated like the ATA program does
//!    (transfer top-up + allocate + assign), so 1 lamport can't grief a launch
//!    (QA-HL-01). Anything with data or a non-system owner is rejected;
//! 3. InitializeMint2: mint authority = PDA, freeze authority = None;
//! 4. create the launch-destination ATA, owned by the `launch_vault` PDA. The
//!    owner is NOT caller-chosen and no instruction signs for it (QA-HL-02);
//! 5. mint exactly 1_000_000_000 * 10^decimals;
//! 6. SetAuthority(MintTokens -> None);
//! 7. re-read the mint and assert supply/authorities; write LaunchConfig.

use anchor_lang::{
    prelude::*,
    system_program::{self, Allocate, Assign, CreateAccount, Transfer},
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken},
    token::{self, spl_token::instruction::AuthorityType, InitializeMint2, Mint, MintTo, SetAuthority, Token},
};

use crate::{
    constants::*,
    error::LaunchError,
    state::LaunchConfig,
    validation::{validate, LaunchParams},
};

#[derive(Accounts)]
pub struct Launch<'info> {
    /// Pays rent. Recorded as `creator`; holds no powers.
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Fresh keypair for the new mint. Must sign. The address may hold lamports
    /// (pre-funding is tolerated) but must be system-owned with no data, so an
    /// existing mint (classic or Token-2022) can never be "onboarded".
    #[account(mut)]
    pub mint: Signer<'info>,

    /// Immutable launch record; `init` => one launch per mint, no re-init.
    #[account(
        init,
        payer = creator,
        space = 8 + LaunchConfig::INIT_SPACE,
        seeds = [LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        bump
    )]
    pub launch_config: Account<'info, LaunchConfig>,

    /// CHECK: data-less PDA (canonical bump verified); temporary mint authority.
    #[account(seeds = [MINT_AUTHORITY_SEED, launch_config.key().as_ref()], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: data-less PDA (canonical bump verified) that owns the launch
    /// destination. Program-derived, NOT caller-chosen; hybrid_launch has no
    /// instruction that signs with it, so nobody can withdraw the supply.
    #[account(seeds = [LAUNCH_VAULT_SEED, mint.key().as_ref(), launch_config.key().as_ref()], bump)]
    pub launch_vault: UncheckedAccount<'info>,

    /// CHECK: must equal ATA(launch_vault, mint, classic Token); created by CPI.
    #[account(mut)]
    pub launch_destination: UncheckedAccount<'info>,

    /// CHECK: the platform fee recipient constant (ADR-013). Must be a plain system account (or not
    /// yet exist, which is also system-owned) and not executable, so fee transfers can always credit it.
    #[account(address = PLATFORM_FEE_RECIPIENT @ LaunchError::FeeRecipientInvalid)]
    pub fee_recipient: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_launch(ctx: Context<Launch>, params: LaunchParams) -> Result<()> {
    let amounts = validate(&params)?;
    {
        let r = ctx.accounts.fee_recipient.to_account_info();
        require_keys_eq!(*r.owner, system_program::ID, LaunchError::FeeRecipientInvalid);
        require!(!r.executable && r.data_is_empty(), LaunchError::FeeRecipientInvalid);
    }
    // The fundability rule uses a conservative rent constant; fail closed if live rent is higher.
    require!(
        Rent::get()?.minimum_balance(CORE_ASSET_SPACE_BYTES) <= CORE_ASSET_RENT_LAMPORTS,
        LaunchError::MintCostConstantStale
    );

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

    // 2. Classic SPL mint account (82 bytes).
    let space = <anchor_spl::token::spl_token::state::Mint as anchor_lang::solana_program::program_pack::Pack>::LEN;
    create_mint_account(&ctx, space, &token_program_id)?;

    // 3. Freeze authority is never set.
    token::initialize_mint2(
        CpiContext::new(token_program_id, InitializeMint2 { mint: ctx.accounts.mint.to_account_info() }),
        params.decimals,
        &ctx.accounts.mint_authority.key(),
        None,
    )?;

    // 4. Launch destination ATA.
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

    // 5 + 6. Mint the fixed supply, then revoke the mint authority.
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
        amounts.total_supply_base,
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

    // 7. Defensive post-conditions.
    {
        let info = ctx.accounts.mint.to_account_info();
        require_keys_eq!(*info.owner, SPL_TOKEN_PROGRAM_ID, LaunchError::PostLaunchCheckFailed);
        let data = info.try_borrow_data()?;
        let mint = Mint::try_deserialize(&mut &data[..])?;
        require!(
            mint.supply == amounts.total_supply_base
                && mint.decimals == params.decimals
                && mint.mint_authority.is_none()
                && mint.freeze_authority.is_none(),
            LaunchError::PostLaunchCheckFailed
        );
        let dest = ctx.accounts.launch_destination.to_account_info();
        require_keys_eq!(*dest.owner, SPL_TOKEN_PROGRAM_ID, LaunchError::PostLaunchCheckFailed);
        let d = anchor_spl::token::TokenAccount::try_deserialize(&mut &dest.try_borrow_data()?[..])?;
        require!(
            d.owner == ctx.accounts.launch_vault.key()
                && d.mint == mint_key
                && d.amount == amounts.total_supply_base
                && d.delegate.is_none()
                && d.close_authority.is_none(),
            LaunchError::PostLaunchCheckFailed
        );
    }

    ctx.accounts.launch_config.set_inner(LaunchConfig {
        version: LAUNCH_CONFIG_VERSION,
        bump: ctx.bumps.launch_config,
        mint_authority_bump: bump,
        creator: ctx.accounts.creator.key(),
        mint: mint_key,
        launch_destination: ctx.accounts.launch_destination.key(),
        launch_vault: ctx.accounts.launch_vault.key(),
        launch_vault_bump: ctx.bumps.launch_vault,
        decimals: params.decimals,
        total_supply_base: amounts.total_supply_base,
        ratio_whole_tokens: params.ratio_whole_tokens,
        ratio_base: amounts.ratio_base,
        collection_size: params.collection_size,
        max_tokens_in_nft_form: amounts.max_tokens_in_nft_form,
        fee_lamports: amounts.fee_lamports,
        fee_recipient: PLATFORM_FEE_RECIPIENT,
        graduation_threshold_lamports: params.graduation_threshold_lamports,
        graduation_slice_pct: amounts.graduation_slice_pct,
        launched_at: Clock::get()?.unix_timestamp,
    });

    msg!(
        "hybrid_launch: mint {} supply {} ratio {} size {}",
        mint_key,
        amounts.total_supply_base,
        params.ratio_whole_tokens,
        params.collection_size
    );
    Ok(())
}

/// Create the mint account. Mirrors the ATA program's handling of pre-funded
/// addresses (QA-HL-01): a system-owned, data-less account that already holds
/// lamports is topped up to rent-exempt, allocated and assigned (the mint
/// keypair signs the tx, so allocate/assign are authorised). Otherwise a plain
/// `create_account`. An account with data or any non-system owner is rejected.
fn create_mint_account(ctx: &Context<Launch>, space: usize, token_program_id: &Pubkey) -> Result<()> {
    let sys = ctx.accounts.system_program.key();
    let mint = ctx.accounts.mint.to_account_info();
    let creator = ctx.accounts.creator.to_account_info();
    let rent_min = Rent::get()?.minimum_balance(space);
    let current = mint.lamports();
    if current == 0 {
        return system_program::create_account(
            CpiContext::new(sys, CreateAccount { from: creator, to: mint }),
            rent_min,
            space as u64,
            token_program_id,
        );
    }
    require_keys_eq!(*mint.owner, system_program::ID, LaunchError::MintAccountInUse);
    require!(mint.data_is_empty() && !mint.executable, LaunchError::MintAccountInUse);
    let top_up = rent_min.saturating_sub(current);
    if top_up > 0 {
        system_program::transfer(CpiContext::new(sys, Transfer { from: creator, to: mint.clone() }), top_up)?;
    }
    system_program::allocate(CpiContext::new(sys, Allocate { account_to_allocate: mint.clone() }), space as u64)?;
    system_program::assign(CpiContext::new(sys, Assign { account_to_assign: mint }), token_program_id)?;
    Ok(())
}
