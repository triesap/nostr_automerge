> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Controlled deviations

No implementation deviation has been taken at package creation. Proposed filenames/API spellings in this handoff are intentionally provisional until their discovery/ADR steps, and resolving them there is not a requirement change.

The executing agent must not skip, merge, reorder or broaden steps unless actual repository evidence proves a step obsolete or unsafe (including a concrete conflict with the approved contract). Before changing the plan, append:

1. Deviation ID, affected step(s), current commit and date.
2. Specific source/test/authority evidence, with paths and exact observed behavior.
3. Why the original step cannot safely be executed as written; distinguish repository drift from convenience.
4. Smallest replacement scope and dependency impact; retain original step IDs with explicit disposition.
5. Durable requirement/ADR impact and unchanged acceptance goals.
6. Required verification and next-step-safety decision.

Unavailable tools or external attestations require accurate blocked/unverified status. They do not justify false passes, weakening acceptance, silently disabling validators or modifying historical evidence. Broadening product scope or external actions needs separate user authority; ordinary within-scope implementation choices belong in the planned decision steps.
