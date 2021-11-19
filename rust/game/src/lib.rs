pub mod utils;

use {
    crate::utils::{
        assert_derivation, assert_initialized, assert_owned_by, create_or_allocate_account_raw,
        get_mask_and_index_for_seq, spl_token_burn, spl_token_mint_to, spl_token_transfer,
        TokenBurnParams, TokenTransferParams,
    },
    anchor_lang::{
        prelude::*,
        solana_program::{
            program::{invoke, invoke_signed},
            program_option::COption,
            program_pack::Pack,
            system_instruction, system_program,
        },
        AnchorDeserialize, AnchorSerialize,
    },
    anchor_spl::token::{Mint, TokenAccount},
    metaplex_token_metadata::instruction::{
        create_master_edition, create_metadata_accounts,
        mint_new_edition_from_master_edition_via_token, update_metadata_accounts,
    },
    spl_token::{
        instruction::{initialize_account2, mint_to},
        state::Account,
    },
};
anchor_lang::declare_id!("p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98");

#[program]
pub mod game {
    use super::*;
}

#[account]
pub struct Network {
    builder_token: Pubkey,
    treasury_wallet: Pubkey,
    authority: Pubkey,
}

#[account]
pub struct World {
    mint: Pubkey,
    metadata: Pubkey,
    edition: Pubkey,
    game: Pubkey,
    player_class: Pubkey,
    /// How many tiles can be crossed per day
    default_traversal_rate: u64,
    default_coin_expenditure_rate: u64,
    default_tile_spawn_rate: u8,
    default_trash_build_rate: u64,
    traversal_token: Pubkey,
}

pub const BUILDING_TYPE_SIZE: usize = 32;
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum BuildingType {
    Cooldown { duration: i64, padding: [u8; 24] },
    Exhaustion { padding: [u8; 32] },
    Destruction { padding: [u8; 32] },
    Infinite { padding: [u8; 32] },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum DistributionType {
    MintOnTheFly,
    FromStorage,
    MasterEditionOnly,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Reward {
    /// class and object identical if this is a playerclass or item class
    class: Pubkey,
    object: Pubkey,
    distribution_type: DistributionType,
    amount: u64,
    base_chance: u8,
}

#[account]
pub struct BuildingClass {
    world: Pubkey,
    building_type: BuildingType,
    /// Building is both a player and an item, item for the creation
    /// and a player for purposes of match
    /// Multiple things happen:
    /// 1. Players enter match with tiles to create building.
    /// Tiles removed, items removed, entry fee paid.  (match 1)
    /// 2. Then rewards rolled. (match 2)
    /// 3. Building is emitted at the end - can be re-explored as player.
    player_class: Pubkey,
    item_class: Pubkey,
    rewards: Vec<Reward>,
    entry_fee: u64,
}

#[account]
pub struct Building {
    parent: Pubkey,
    mint: Pubkey,
    metadata: Pubkey,
    edition: Pubkey,
    item: Pubkey,
    player: Pubkey,
    rewards: Vec<Reward>,
}

#[account]
pub struct MapTileMasterEdition {
    mint: Pubkey,
    metadata: Pubkey,
    edition: Pubkey,
    player_class: Pubkey,
    world: Pubkey,
}

#[account]
pub struct MapTileEdition {
    mint: Pubkey,
    metadata: Pubkey,
    edition: Pubkey,
    player: Pubkey,
    parent: Pubkey,
}

#[error]
pub enum ErrorCode {
    #[msg("Account does not have correct owner!")]
    IncorrectOwner,
    #[msg("Account is not initialized!")]
    Uninitialized,
    #[msg("Mint Mismatch!")]
    MintMismatch,
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    #[msg("Numerical overflow error")]
    NumericalOverflowError,
    #[msg("Token mint to failed")]
    TokenMintToFailed,
    #[msg("TokenBurnFailed")]
    TokenBurnFailed,
    #[msg("Derived key is invalid")]
    DerivedKeyInvalid,
}
