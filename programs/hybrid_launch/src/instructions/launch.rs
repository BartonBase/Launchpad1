//! `launch`: create a Track A classic SPL Token mint with a fixed 1B supply and
//! an immutable LaunchConfig, in ONE instruction:
//! 1. validate params (ratio set, collection_size * ratio <= 1B via checked_mul,
//!    fee caps, capture fee >= re-roll fee, fee destination = BURN);
//! 2. create the 82-byte mint account owned by the CLASSIC Token program
//!    (`token_program: Program<Token>` rejects Token-2022: INV-13);
//! 3. InitializeMint2: mint authority = PDA, freeze authority = None;
//! 4. create the launch-destination ATA;
//! 5. mint exactly 1_000_000_000 * 10^decimals;
//! 6. SetAuthority(MintTokens -> None);
//! 7. re-read the mint and assert supply/authorities; write LaunchConfig.

use anchor_lang::{
    prelude::*,
    system_program::{self, CreateAccount},
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

    /// Fresh keypair for the new mint. Must sign; `create_account` fails if the
    /// address already holds an account (so an existing mint, classic or
    /// Token-2022, can never be "onboarded").
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

    /// CHECK: owner of the account receiving the full supply (future: curve vault).
    pub launch_destination_owner: UncheckedAccount<'info>,

    /// CHECK: must equal ATA(launch_destination_owner, mint, classic Token); created by CPI.
    #[account(mut)]
    pub launch_destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_launch(ctx: Context<Launch>, params: LaunchParams) -> Result<()> {
    let amounts = validate(&params)?;

    let token_program_id = ctx.accounts.token_program.key();
    let mint_key = ctx.accounts.mint.key();
    let config_key = ctx.accounts.launch_config.key();

    require_keys_eq!(
        ctx.accounts.launch_destination.key(),
        get_associated_token_address_with_program_id(
            &ctx.accounts.launch_destination_owner.key(),
            &mint_key,
            &token_program_id
        ),
        LaunchError::InvalidLaunchDestination
    );

    // 2. Classic SPL mint account (82 bytes).
    let space = <anchor_spl::token::spl_token::state::Mint as anchor_lang::solana_program::program_pack::Pack>::LEN;
    system_program::create_account(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            CreateAccount {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.mint.to_account_info(),
            },
        ),
        Rent::get()?.minimum_balance(space),
        space as u64,
        &token_program_id,
    )?;

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
            authority: ctx.accounts.launch_destination_owner.to_account_info(),
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
    }

    ctx.accounts.launch_config.set_inner(LaunchConfig {
        version: LAUNCH_CONFIG_VERSION,
        bump: ctx.bumps.launch_config,
        mint_authority_bump: bump,
        creator: ctx.accounts.creator.key(),
        mint: mint_key,
        launch_destination: ctx.accounts.launch_destination.key(),
        decimals: params.decimals,
        total_supply_base: amounts.total_supply_base,
        ratio_whole_tokens: params.ratio_whole_tokens,
        ratio_base: amounts.ratio_base,
        collection_size: params.collection_size,
        max_tokens_in_nft_form: amounts.max_tokens_in_nft_form,
        capture_fee_bps: params.capture_fee_bps,
        capture_fee_amount: amounts.capture_fee_amount,
        reroll_fee_bps: params.reroll_fee_bps,
        reroll_fee_amount: amounts.reroll_fee_amount,
        fee_destination: FEE_DESTINATION_BURN,
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
