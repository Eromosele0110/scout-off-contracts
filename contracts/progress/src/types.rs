use soroban_sdk::{contracttype, Address};

pub use scoutchain_shared_types::{ContractHealth, MigrationStatus, ProgressLevel};

/// Storage layout version this build of the contract expects.
///
/// Bump this only when the *layout* changes — a new `DataKey` variant, a changed
/// field type, or a different meaning for an existing key. Adding a new
/// entrypoint that writes keys of its own does not require a bump, because
/// older readers ignore keys they do not know about.
///
/// The migration from 0 to 1 backfills [`DataKey::HistoryVec`] for players
/// registered before that key existed.
pub const CODE_SCHEMA_VERSION: u32 = 1;

/// A single entry in the immutable progress history
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProgressEntry {
    /// Unique player identifier whose level changed.
    pub player_id: u64,
    /// Player level before this history entry was recorded.
    pub old_level: ProgressLevel,
    /// Player level after this history entry was recorded.
    pub new_level: ProgressLevel,
    /// Wallet that triggered the update (validator or scout)
    pub updated_by: Address,
    /// Ledger timestamp when the level change was recorded, in Unix seconds.
    pub updated_at: u64,
    /// Milestone index from the verification contract that triggered this
    pub milestone_ref: u32,
    /// Ledger sequence number at the time of the level change
    pub ledger_sequence: u32,
}

#[contracttype]
pub enum DataKey {
    /// The `Address` of the contract administrator. Set during `initialize` and
    /// updated by `accept_admin`. Required for all privileged operations.
    Admin,
    /// Proposed replacement admin. The address stored here must call
    /// `accept_admin` before `Admin` is updated.
    PendingAdmin,
    /// Boolean flag (`true`) written during `initialize`. Absence or `false`
    /// means the contract has not yet been set up; `health()` reads this key.
    Initialized,
    /// Boolean flag indicating whether the contract is currently paused.
    /// `true` blocks all state-changing operations; `false` allows them.
    /// Toggled by `pause_contract` / `unpause_contract`.
    Paused,
    /// Maps a `player_id` (`u64`) to the player's current [`ProgressLevel`].
    /// Absent until the player's first level advancement; defaults to
    /// [`ProgressLevel::Unverified`] when read.
    PlayerLevel(u64),
    /// Tracks the total number of history entries recorded for a given
    /// `player_id`. Acts as a monotonically increasing counter; the current
    /// value is also the index of the most-recent [`HistoryEntry`].
    HistoryCounter(u64),
    /// Stores a [`ProgressEntry`] for a specific `(player_id, history_index)`
    /// pair. Indices start at `1` and are assigned by [`HistoryCounter`].
    HistoryEntry(u64, u32),
    /// Stores **all** history entries for a player as a single `Vec<ProgressEntry>`.
    /// Reading this key costs one persistent storage read regardless of entry count,
    /// replacing the O(N) loop in `get_progress_history`. Written in parallel with
    /// [`HistoryEntry`] so both access patterns remain valid.
    HistoryVec(u64),
    /// The `Address` of the companion verification contract. Reserved for
    /// future cross-contract authorisation checks; not yet written at runtime.
    VerificationContract,
    /// The `Address` of the registration contract. Only this address is
    /// permitted to call `initialize_player`. Set by `set_registration_contract`.
    RegistrationContract,
    /// The `Address` of the scout_access contract. Whitelisted as a secondary
    /// authorised caller of `advance_level` (for trial-offer Level-3 advances).
    ScoutAccessContract,
    /// The storage layout version this contract is currently running. Absent
    /// means version 0 — the pre-versioning layout — so a contract that has
    /// never been migrated reads as "behind the code" rather than "current".
    SchemaVersion,
    /// How far a resumable migration has progressed. Stores the highest
    /// `player_id` the cursor has already visited, so a second `migrate` call
    /// resumes instead of re-scanning from zero. Absent means "not started".
    MigrationCursor(u64),
    /// Total items rewritten by a migration across all calls. Diagnostic only:
    /// it lets an operator confirm progress across several `migrate` calls
    /// without reading the cursor's implied position.
    MigrationProcessed,
}
