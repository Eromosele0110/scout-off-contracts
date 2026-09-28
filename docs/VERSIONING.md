# Storage schema versioning

How a deployed contract knows which storage layout it is running, and how to
move a contract to a new one.

## Why this exists

`upgrade()` replaces the contract's WASM immediately. Soroban gives a contract
no hook that runs *after* its own code has been swapped, so a contract that
upgrades from a layout-1 build to a layout-2 build starts serving traffic
against a layout-2 contract while its storage is still layout-1.

Until now that was only detectable from outside, by
`scripts/check-storage-layout-compat.sh` and manual replay. The contract itself
could not say which layout it was on, so a deployment could not tell a
successful upgrade from a silent one.

## The version

`CODE_SCHEMA_VERSION` is a compile-time constant in each contract's `types.rs`,
paired with a `DataKey::SchemaVersion` in instance storage.

- **Absent `SchemaVersion` means version 0**, the pre-versioning layout. Reading
  it as "current" would let a contract that had never been migrated look
  healthy.
- `initialize` records the current version, because a fresh contract has no
  older state to migrate.
- `schema_version()` is a view over the stored value.
- `health()` is deliberately unchanged: `ContractHealth` is shared across all
  four contracts, and changing its shape would break every caller for the sake
  of one field. `MigrationStatus` is reported by `migrate()` instead.

**Bump the constant only when the layout changes** — a new `DataKey` variant, a
changed field type, or a different meaning for an existing key. Adding an
entrypoint that writes keys of its own does not require a bump, because older
readers ignore keys they do not know about.

## Migrating

```rust
migrate(target_version: u32, max_items: u32) -> Result<MigrationStatus, ProgressError>
```

Admin-only, and bounded by `max_items` per call. A full history backfill can
exceed what one transaction can afford, so each call does a slice of the work
and records a cursor in `DataKey::MigrationCursor`. Call it repeatedly until
`complete` is true.

| Situation | Behaviour |
| --- | --- |
| `target` above stored version | runs the migration, emits `schema_migrated(from, to)` |
| `target` equals stored version | no-op, reports `complete`, rewrites nothing |
| `target` **below** stored version | `SchemaVersionTooNew` — refused, never rolled back |
| `target` above `CODE_SCHEMA_VERSION` | `UnknownSchemaTarget` |
| `max_items == 0` | no work, cursor unmoved |

Two of these deserve explanation.

**Idempotence is what makes a retrying upgrade script safe.** A deployment that
half-fires and retries will call `migrate` against storage that is already
current. That is a no-op that reports `complete`, not an error — and it does not
re-emit `schema_migrated`, so one logical migration produces one event.

**A downgrade is refused rather than rolled back.** Moving storage to an older
layout would discard data the running code expects. Refusing loudly is the only
safe answer, and it also catches a genuinely wrong target early.

**`max_items == 0` is a no-op on purpose.** Treating it as "no limit" would
silently do the entire backfill in one transaction; treating it as "done" would
mark an unmigrated contract as migrated. It advances nothing, so the caller's
loop runs again.

## Procedure

`scripts/upgrade.sh` drives this. It uploads the new WASM, calls `upgrade()`,
then loops `migrate()` until `complete`, and finally verifies through
`schema_version()`:

```bash
./scripts/upgrade.sh --contract <id> --wasm <path> --network testnet
```

## Current migrations

### progress: v0 → v1

Backfills `DataKey::HistoryVec` for players registered before that key existed.
`HistoryEntry(player, index)` is already correct, so the migration copies rather
than recomputes — there is no way for the two to disagree afterwards.

Players with no history entries are skipped but still counted against
`max_items`, and the cursor advances past them. Reconsidering an empty player on
every subsequent call is what would stop a repeated migration from terminating.

Covered by `test_migrate_*` and `test_upgrade_preserves_schema_version_key` in
`contracts/progress/src/lib.rs`, which use `simulate_v0_contract` to rewind
storage to the real pre-upgrade state rather than asserting against a mock.

## Adding a migration

1. Bump `CODE_SCHEMA_VERSION`.
2. Add the step to `migrate()`, keyed on `from < N && target_version >= N`.
3. Add a `simulate_v{n}_contract` helper that produces the old layout.
4. Add rehearsal tests: one step of real work, resume-across-calls, and
   idempotence.
5. Update this file.
