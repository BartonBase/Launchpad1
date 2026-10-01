//! `launch_token22` (Mode 4): Token-2022 mint with a transfer tax, fixed 1B supply, in ONE instruction.
//! The tax rate is chosen here and the fee-config authority is None, so nobody can change it.
//! The withdraw authority is the vault program's tax PDA. This path does not burn.
//! an immutable T22BurnLaunchConfig, in ONE instruction:
//! 1. validate params (ratio set, 100 <= collection_size, collection_size * ratio <= 1B via
//!    checked_mul, collection_size <= MAX_COLLECTION_SIZE; the flat SOL fee is looked up from the
//!    ratio tier table, never supplied; there is no token fee, ADR-013);
//! 2. create the base-length mint account owned by the Token-2022 program.
//!    No extension account is initialized. `Program<Token2022>` rejects classic SPL. Pre-funded
//!    but empty system accounts are tolerated like the ATA program does
//!    (transfer top-up + allocate + assign), so 1 lamport can't grief a launch
//!    (QA-HL-01). Anything with data or a non-system owner is rejected;
//! 3. InitializeMint2: mint authority = PDA, freeze authority = None;
//! 4. create the launch-destination ATA, owned by the `launch_vault` PDA. The
//!    owner is NOT caller-chosen and no instruction signs for it (QA-HL-02);
//! 5. mint exactly 1_000_000_000 * 10^decimals;
//! 6. SetAuthority(MintTokens -> None);
//! 7. re-read the mint and assert supply/authorities; write T22BurnLaunchConfig.
//! No transfer fee, freeze, permanent delegate, or transfer hook. Modes 1-3 are unchanged.

use anchor_lang::{
    prelude::*,
    system_program::{self, Allocate, Assign, CreateAccount, Transfer},
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken},
    token_2022::{
        self as token_2022, spl_token_2022, InitializeMint2, MintTo, SetAuthority, Token2022,
    },
};

use crate::{
    constants::*,
    error::LaunchError,
    state::T22BurnLaunchConfig,
    validation::{validate, LaunchParams},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TaxLaunchParams {
    pub decimals: u8,
    pub ratio_whole_tokens: u64,
    pub collection_size: u64,
    pub graduation_threshold_lamports: u64,
    /// Transfer tax in basis points. 1..=1000. Locked by a None config authority.
    pub tax_bps: u16,
    /// Lamports per whole token for the inventory buyback. 1..=1 SOL. Locked at launch.
    pub buyback_lamports_per_whole: u64,
}

#[derive(Accounts)]
pub struct LaunchToken22<'info> {
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
        space = 8 + T22BurnLaunchConfig::INIT_SPACE,
        seeds = [LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        bump
    )]
    pub launch_config: Account<'info, T22BurnLaunchConfig>,

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

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_launch_token22(ctx: Context<LaunchToken22>, params: TaxLaunchParams) -> Result<()> {
    handle_launch_tax(ctx, params, LAUNCH_MODE_TOKEN22)
}

pub fn handle_launch_tax(
    ctx: Context<LaunchToken22>,
    params: TaxLaunchParams,
    launch_mode: u8,
) -> Result<()> {
    require!(
        launch_mode == LAUNCH_MODE_TOKEN22 || launch_mode == LAUNCH_MODE_RAFFLE,
        LaunchError::PostLaunchCheckFailed
    );
    require!(
        (MIN_TAX_BPS..=MAX_TAX_BPS).contains(&params.tax_bps),
        LaunchError::TaxBpsNotAllowed
    );
    require!(
        (MIN_BUYBACK_LAMPORTS_PER_WHOLE..=MAX_BUYBACK_LAMPORTS_PER_WHOLE)
            .contains(&params.buyback_lamports_per_whole),
        LaunchError::BuybackPriceNotAllowed
    );
    let lp = LaunchParams {
        decimals: params.decimals,
        ratio_whole_tokens: params.ratio_whole_tokens,
        collection_size: params.collection_size,
        graduation_threshold_lamports: params.graduation_threshold_lamports,
    };
    let amounts = validate(&lp)?;
    {
        let r = ctx.accounts.fee_recipient.to_account_info();
        require_keys_eq!(
            *r.owner,
            system_program::ID,
            LaunchError::FeeRecipientInvalid
        );
        require!(
            !r.executable && r.data_is_empty(),
            LaunchError::FeeRecipientInvalid
        );
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

    // Token-2022 mint sized for TransferFeeConfig only.
    let space = crate::t22::tax_mint_len();
    create_token22_mint_account(&ctx, space, &token_program_id)?;

    let (tax_authority, _) =
        Pubkey::find_program_address(&[TAX_AUTHORITY_SEED, mint_key.as_ref()], &HYBRID_VAULT_ID);
    let init_fee = anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::instruction::initialize_transfer_fee_config(
        &token_program_id,
        &mint_key,
        None,
        Some(&tax_authority),
        params.tax_bps,
        amounts.total_supply_base,
    )?;
    anchor_lang::solana_program::program::invoke(
        &init_fee,
        &[ctx.accounts.mint.to_account_info()],
    )?;

    // Freeze authority is never set. Fee-config authority was never set, so the rate is locked.
    token_2022::initialize_mint2(
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
    token_2022::mint_to(
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
    token_2022::set_authority(
        CpiContext::new_with_signer(
            token_program_id,
            SetAuthority {
                current_authority: ctx.accounts.mint_authority.to_account_info(),
                account_or_mint: ctx.accounts.mint.to_account_info(),
            },
            signer,
        ),
        spl_token_2022::instruction::AuthorityType::MintTokens,
        None,
    )?;

    // 7. Defensive post-conditions.
    {
        let info = ctx.accounts.mint.to_account_info();
        require_keys_eq!(
            *info.owner,
            spl_token_2022::ID,
            LaunchError::PostLaunchCheckFailed
        );
        let data = info.try_borrow_data()?;
        let mint = crate::t22::read_tax_mint(&data)?;
        require!(
            mint.supply == amounts.total_supply_base
                && mint.decimals == params.decimals
                && mint.mint_authority_none
                && mint.freeze_authority_none
                && mint.config_authority_none
                && mint.tax_bps == params.tax_bps
                && mint.maximum_fee == amounts.total_supply_base
                && mint.withdraw_authority == tax_authority,
            LaunchError::PostLaunchCheckFailed
        );
        let dest = ctx.accounts.launch_destination.to_account_info();
        require_keys_eq!(
            *dest.owner,
            spl_token_2022::ID,
            LaunchError::PostLaunchCheckFailed
        );
        let d = crate::t22::read_token_account(&dest.try_borrow_data()?)?;
        require!(
            d.owner == ctx.accounts.launch_vault.key()
                && d.mint == mint_key
                && d.amount == amounts.total_supply_base
                && d.delegate_none
                && d.close_none,
            LaunchError::PostLaunchCheckFailed
        );
    }

    // QA-FEE-03: the stored fee must be exactly the tier (shared derivation), never 0 or off-tier.
    require!(
        crate::validation::is_exact_tier_fee(amounts.fee_lamports, params.ratio_whole_tokens),
        LaunchError::FeeNotTier
    );
    ctx.accounts.launch_config.set_inner(T22BurnLaunchConfig {
        version: T22_BURN_LAUNCH_CONFIG_VERSION,
        bump: ctx.bumps.launch_config,
        mint_authority_bump: bump,
        launch_mode,
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
        dbc_config: Pubkey::default(),
        dbc_pool: Pubkey::default(),
        tax_bps: params.tax_bps,
        tax_authority,
        buyback_lamports_per_whole: params.buyback_lamports_per_whole,
    });

    msg!(
        "hybrid_launch: token22 mint {} tax {}bps supply {}",
        mint_key,
        params.tax_bps,
        amounts.total_supply_base
    );
    Ok(())
}

/// Create the mint account. Mirrors the ATA program's handling of pre-funded
/// addresses (QA-HL-01): a system-owned, data-less account that already holds
/// lamports is topped up to rent-exempt, allocated and assigned (the mint
/// keypair signs the tx, so allocate/assign are authorised). Otherwise a plain
/// `create_account`. An account with data or any non-system owner is rejected.
fn create_token22_mint_account(
    ctx: &Context<LaunchToken22>,
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

/// Spend SOL already on the tax PDA to buy tokens from this launch's inventory.
/// Destination is the tax treasury ATA only. Price is the value stored at launch.
#[derive(Accounts)]
pub struct BuyInventory<'info> {
    #[account(
        mut,
        seeds = [LAUNCH_CONFIG_SEED, mint.key().as_ref()],
        bump = launch_config.bump,
        has_one = mint,
        has_one = launch_vault,
        has_one = launch_destination,
        has_one = tax_authority,
    )]
    pub launch_config: Box<Account<'info, T22BurnLaunchConfig>>,

    /// CHECK: Token-2022 mint created by this launch.
    pub mint: UncheckedAccount<'info>,

    /// CHECK: inventory owner. Signs the token sale only, and only to the tax treasury.
    #[account(
        mut,
        seeds = [LAUNCH_VAULT_SEED, mint.key().as_ref(), launch_config.key().as_ref()],
        bump = launch_config.launch_vault_bump
    )]
    pub launch_vault: UncheckedAccount<'info>,

    /// CHECK: launch inventory ATA.
    #[account(mut)]
    pub launch_destination: UncheckedAccount<'info>,

    /// CHECK: ATA(tax_authority, mint). Not caller-chosen.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    /// Vault tax PDA. Must sign. Only hybrid_vault can. Writable because it pays the SOL.
    #[account(mut)]
    pub tax_authority: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handle_buy_inventory(ctx: Context<BuyInventory>, sol_amount: u64) -> Result<()> {
    let cfg = &ctx.accounts.launch_config;
    require!(
        cfg.launch_mode == LAUNCH_MODE_TOKEN22 || cfg.launch_mode == LAUNCH_MODE_RAFFLE,
        LaunchError::PostLaunchCheckFailed
    );
    let price = cfg.buyback_lamports_per_whole;
    require!(price > 0, LaunchError::BuybackPriceNotAllowed);
    let token_program_id = ctx.accounts.token_program.key();
    require_keys_eq!(
        ctx.accounts.treasury.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.tax_authority.key(),
            &ctx.accounts.mint.key(),
            &token_program_id
        ),
        LaunchError::InvalidLaunchDestination
    );
    let inventory =
        crate::t22::read_token_account(&ctx.accounts.launch_destination.try_borrow_data()?)
            .map_err(|_| error!(LaunchError::ExtensionsNotAllowed))?;
    require!(
        inventory.owner == cfg.launch_vault && inventory.mint == cfg.mint,
        LaunchError::PostLaunchCheckFailed
    );
    let unit = 10u64
        .checked_pow(cfg.decimals as u32)
        .ok_or(LaunchError::MathOverflow)?;
    let whole_cap = inventory.amount / unit;
    let whole = sol_amount
        .checked_div(price)
        .ok_or(LaunchError::MathOverflow)?;
    let whole = whole.min(whole_cap);
    if whole == 0 {
        return Ok(());
    }
    let base = whole.checked_mul(unit).ok_or(LaunchError::MathOverflow)?;
    let sol_spent = whole.checked_mul(price).ok_or(LaunchError::MathOverflow)?;
    let epoch = Clock::get()?.epoch;
    let fee = crate::t22::transfer_fee(&ctx.accounts.mint.try_borrow_data()?, epoch, base)?;
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.tax_authority.to_account_info(),
                to: ctx.accounts.launch_vault.to_account_info(),
            },
        ),
        sol_spent,
    )?;
    let bump = cfg.launch_vault_bump;
    let mint_key = ctx.accounts.mint.key();
    let config_key = ctx.accounts.launch_config.key();
    let seeds: &[&[u8]] = &[
        LAUNCH_VAULT_SEED,
        mint_key.as_ref(),
        config_key.as_ref(),
        &[bump],
    ];
    let ix = spl_token_2022::extension::transfer_fee::instruction::transfer_checked_with_fee(
        &token_program_id,
        &ctx.accounts.launch_destination.key(),
        &mint_key,
        &ctx.accounts.treasury.key(),
        &ctx.accounts.launch_vault.key(),
        &[],
        base,
        cfg.decimals,
        fee,
    )?;
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.launch_destination.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.treasury.to_account_info(),
            ctx.accounts.launch_vault.to_account_info(),
        ],
        &[seeds],
    )?;
    Ok(())
}
