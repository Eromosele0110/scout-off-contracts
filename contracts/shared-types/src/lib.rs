#![no_std]
use soroban_sdk::{contracttype, String};

/// Four-tier progress level for a player profile
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProgressLevel {
    /// Level 0 — profile created, no verification yet
    Unverified,
    /// Level 1 — identity confirmed by academy or KYC
    VerifiedIdentity,
    /// Level 2 — performance milestones verified by approved third party
    PerformanceMilestones,
    /// Level 3 — scout feedback or trial offer logged
    EliteTier,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ContractHealth {
    pub initialized: bool,
    pub paused: bool,
}

/// Progress of a bounded, resumable storage migration.
///
/// A migration is reported rather than hidden: an operator has to be able to
/// tell "not started" from "half done" from "finished", because `migrate` is
/// expected to be called repeatedly until `complete` is true.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MigrationStatus {
    /// Schema version the contract was on when this call started.
    pub from: u32,
    /// Schema version this call targeted.
    pub to: u32,
    /// Schema version compiled into the running WASM.
    pub code: u32,
    /// Schema version currently recorded in storage.
    pub current: u32,
    /// True while storage is still behind the code's layout.
    pub pending: bool,
    /// True once `current` has reached `code`.
    pub complete: bool,
    /// Highest id the cursor has already visited. `0` means "not started".
    pub last_visited_id: u64,
    /// Total items rewritten across all calls so far.
    pub processed: u32,
}

impl ProgressLevel {
    /// Returns the next valid level, or None if already at the top.
    pub fn next(&self) -> Option<ProgressLevel> {
        match self {
            ProgressLevel::Unverified => Some(ProgressLevel::VerifiedIdentity),
            ProgressLevel::VerifiedIdentity => Some(ProgressLevel::PerformanceMilestones),
            ProgressLevel::PerformanceMilestones => Some(ProgressLevel::EliteTier),
            ProgressLevel::EliteTier => None,
        }
    }
}

/// Validate that a string is a plausible IPFS/Arweave CID.
///
/// Rules:
/// - CIDv0: starts with "Qm", exactly 46 characters, base58btc charset
///   (no 0, O, I, l characters).
/// - CIDv1 (base32): starts with "bafy", 59–128 characters.
pub fn validate_cid(hash: &String) -> Result<(), &'static str> {
    let hash_len = hash.len();
    let bytes = hash.to_bytes();

    let starts_with_qm = bytes.get(0) == Some(b'Q') && bytes.get(1) == Some(b'm');
    let starts_with_bafy = hash_len >= 4
        && bytes.get(0) == Some(b'b')
        && bytes.get(1) == Some(b'a')
        && bytes.get(2) == Some(b'f')
        && bytes.get(3) == Some(b'y');

    if starts_with_qm {
        // CIDv0: exactly 46 chars
        if hash_len != 46 {
            return Err("invalid cid: CIDv0 must be exactly 46 characters");
        }
        // Base58btc: no 0, O, I, l
        for i in 0..hash_len {
            match bytes.get(i) {
                Some(b'0') | Some(b'O') | Some(b'I') | Some(b'l') => {
                    return Err("invalid cid: CIDv0 contains invalid base58btc character");
                }
                _ => {}
            }
        }
        Ok(())
    } else if starts_with_bafy {
        // CIDv1 (base32): 59–128 chars
        if !(59..=128).contains(&hash_len) {
            return Err("invalid cid: CIDv1 must be 59–128 characters");
        }
        Ok(())
    } else {
        Err("invalid cid: must start with 'Qm' (CIDv0) or 'bafy' (CIDv1)")
    }
}
