# Future Farm Workspaces context

This file preserves downstream product intent. It does not define generic
`nostr_automerge` validity or initial repository scope.

## Product

Private, offline-capable operations workspace for a 2–10-person small market
farm with multiple sites/locations.

## Highest-leverage workflows

- Today plan: Must / Should / If time.
- Work assignment, claim, start, block, checklist, completion, verification.
- Recurring irrigation, greenhouse, cooler, sanitation, equipment, market,
  animal, and shutdown work.
- Quick pest/disease/water/equipment/facility/material/safety observations.
- Harvest runs with target, source, destination, cutoff, pack spec, actual,
  rejection, shortage, cooler/loading state.
- Structured shift handoff.
- Private time and correction evidence.
- Private compensation/settlement records.

## Document topology

### workspace_root

Readers: active team.
Writers: owner/managers.

Contains locations, directory, templates, active operations pointer.

### operations_period

Readers: active team.
Writers: crew/managers.

Contains shared high-frequency work state.

### admin_vault

Readers/writers: explicit administrators.

Contains ledger directory, scopes, policies, retention, admin sagas.

### worker_ledger

Readers: worker and selected administrators.
Writers: worker and ledger stewards.

Contains agreements, hours, corrections, in-kind entries, bonuses, disputes,
settlements.

## Compensation forms

- hourly cash;
- fixed shift/day/week/project, with actual hours retained;
- piece/production rate only under reviewed policy;
- produce or CSA credit with quantity/value/acknowledgment;
- approved reimbursement;
- seasonal/retention/quality/team bonus;
- allocation of externally finalized revenue/profit pool;
- ownership/cooperative/crop-share/partnership structures kept separate from
  ordinary wages.

## Future architecture

- shared Rust application core;
- one UniFFI boundary for SwiftUI iOS and future Kotlin Android;
- official Marmot MDK for private MLS delivery;
- Tangle hosted virtual tenants;
- optional LAN edge Tangle relay;
- direct nearby exact-event synchronization;
- NFC for pairing/location/equipment tags, not bulk transfer.

## Invariant

All downstream paths consume the generic `nostr_automerge` implementation.
They do not reimplement ActorId derivation, change framing, control selection,
equivocation, checkpoint verification, or canonical digests.
