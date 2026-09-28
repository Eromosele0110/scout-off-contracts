use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ProgressError {
    /// Contract has already been initialized and cannot be initialized again.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet; call `initialize` first.
    NotInitialized = 2,
    /// Contract is paused; all state-changing operations are blocked.
    ContractPaused = 3,
    /// Caller is not authorized to perform this operation.
    Unauthorized = 4,
    /// The requested level transition is not valid (e.g. skipping a level or going backwards).
    InvalidProgressTransition = 5,
    /// Player is already at the maximum level (EliteTier) and cannot advance further.
    AlreadyAtMaxLevel = 6,
    /// No progress record exists for the given player ID.
    PlayerNotFound = 7,
    /// History counter overflowed the maximum u32 value.
    Overflow = 8,
    /// Call to registration contract failed.
    RegistrationCallFailed = 9,
    /// The stored schema version is newer than the version this build expects.
    /// The contract was downgraded, and running a migration would corrupt state
    /// written by a later layout. Refused rather than silently ignored.
    SchemaVersionTooNew = 10,
    /// `migrate` was called with a target version the contract does not know
    /// how to reach.
    UnknownSchemaTarget = 11,
}
