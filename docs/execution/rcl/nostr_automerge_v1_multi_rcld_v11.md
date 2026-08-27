# nostr_automerge draft-v1 remediation v11 multi-RCLD plan

Status: `code_complete_publication_held`

The reviewed public predecessor is
`e1b4f461c0d2a1e8cc8e520bed2dfa64a62270f2`. The approved opaque independent
compatibility predecessor is `2d708bb0a7a00523ab5c244fd0a15c96afcf0a4a`.
Steps `step_1308` through `step_1363` are 56 contiguous checkpoints. No remote
action is authorized.

| RCLD | Checkpoints | Lane | Exit condition |
| --- | --- | --- | --- |
| 100 | `step_1308`–`step_1314` | Authority and reproductions | Baselines, Findings 096–099, Finding 080, ADRs 0072–0075, supersession, and exact open reproductions are bound without a runtime fix. |
| 101 | `step_1315`–`step_1320` | Rust persistent-state core | Lookup, membership, and extension are metered and cancellable with no runtime bypass. |
| 102 | `step_1321`–`step_1326` | Rust persistent-state integration | Every production persistent-state caller uses the metered boundary and ample-capacity output remains compatible. |
| 103 | `step_1327`–`step_1334` | Rust complete target-work accounting | Every remaining proportional operation has one live, constant, or reserved owner and no post-stop work remains. |
| 104 | `step_1335`–`step_1339` | Rust bounded persistent teardown | Deep unique chains and wide shared forks tear down on a constrained stack. |
| 105 | `step_1340`–`step_1345` | Public authority reconciliation | NIP, companion, API, requirements, ADRs, and distribution-v12 authority agree. |
| 106 | `step_1346`–`step_1351` | Private compatibility parity | The independent implementation satisfies immutable metered state and emits only approved opaque evidence. |
| 107 | `step_1352`–`step_1358` | Distribution-v12 parity | Exactly 198 signed scenarios pass eight delivery orders and two processes per implementation. |
| 108 | `step_1359`–`step_1363` | Evidence and final assurance | Findings 096–099 close locally, Finding 080 remains held, and the terminal status is code complete with publication held. |

## Execution invariants

- Execute checkpoints in numerical order with one active checkpoint.
- A red checkpoint is repaired, split, or blocked and is never committed.
- The caller meters local-delta preparation; persistent state meters inherited
  duplicate checks and accepted insertions.
- Persistent lookup counts exact nodes actually visited and samples
  cancellation before each visit.
- Existing qualified finalization reservations remain exact. Newly audited
  live work is charged immediately before its owned operation.
- Five new signed scenarios are representative; exhaustive boundaries remain
  in source tests and generation assertions.
- Public and independent private histories remain separate.
- Publication, release, deployment, remote mutation, NIP submission,
  event-kind allocation, production qualification, and external assurance are
  held throughout this plan.

## Completion rule

An RCLD is unfinished until every checkpoint in its inclusive range has a green
committed candidate and fresh validation. Local completion never implies
publication or external assurance.

RCLDs 100 through 108 are locally complete. Finding 080 and every external,
allocation, submission, production-qualification, publication, release, and
remote-mutation hold remain in force.
