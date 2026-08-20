# Remediation V8 Local NIP Reconciliation

Status: complete
Scope: repository-local draft only; no submission, allocation, publication, or
remote action is authorized.

## Rebase identity

- `spec/NIP_DRAFT.md` before reconciliation:
  `67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3`
- `spec/NOSTR_AUTOMERGE_V1_SPEC.md` before reconciliation:
  `5ab298eb06399e1dc1631898f1910c3a33860b8b40b2c0cf9b7f2266fdf23`
- `spec/CONFORMANCE.md` before reconciliation:
  `28c3aff44d333895c2f6dfe1b4dd83563e0fcb4c47c1b5d07d16487c7e3e3807`
- `spec/requirements.json` before reconciliation:
  `2432157fc76c70c1833210d7bac3ea410f5071f0ce6b0246bb59fbfefdee52c3`

## Section delta

The approved v8 reconciliation affects the draft sections `Actor sequence and
operation counters`, `Canonical control chain`, `Change event`, `Duplicate
carriers`, `Deterministic state construction`, `Protocol dispositions and
local completion`, `Parsing and resource use`, and `Conformance`. It also
reconciles the companion's status, dynamic event dispositions, coordinate
scope, and finalization rules.

The prior companion describes the NIP as an immutable external snapshot and
keeps reconciliation as an external hold. The approved v8 authority supersedes
that local coordination rule only for this repository-local draft. It does not
authorize an upstream NIP edit or any publication claim.

## Preserved wire and status constants

- The title remains `NIP-XX`, with `draft` and `optional` status.
- Event kinds remain provisional `1624`, `1625`, `1626`, `1627`, and `31624`.
- The document coordinate remains `31624:<controller-pubkey>:<document-id>`.
- ActorId derivation, required tags, canonical JSON, base64, Automerge framing,
  sealed limits, and event content fields remain unchanged.
- History and dispositions digest domains, namespace codes, and identifier
  ordering remain unchanged.
- The report schema remains `nostr_automerge.report.v1`.
- No relay behavior, acquisition behavior, application schema, or public API
  is added by the reconciliation.

## Reconciliation constraints

The draft must state canonical behavior without naming Rust or TypeScript
implementation structures. The companion may provide implementation detail,
but it must agree with the draft. Every changed authority anchor and bound hash
must be regenerated after the prose is final. Distribution-v9 and private
parity bindings remain pending until RCLD 79.

## Independent implementer-language review

The reconciled draft was reread as a language-neutral protocol contract without
relying on Rust or TypeScript structure names. An implementer can derive all of
the following directly from the draft:

- the causal operation-counter formula and checked-overflow behavior;
- stateful evaluation against each control's actual parent branch;
- branch-local change outcomes and valid-carrier aggregate dominance;
- distinct ChangeHash and carrier Event records, ordering, and digest input;
- coordinate-qualified evidence, indexes, work, and report attribution;
- complete control and descriptor reference-state distinctions;
- pass-level finalization consumption, refund, forfeiture, and fallback; and
- the exact signed-v9 two-run, eight-permutation, mismatch-rejection contract.

No implementation type, private path, or operator-only mechanism is needed to
interpret those rules. The companion retains optimization and pressure-test
detail but defines no contradictory canonical behavior.

## Final identities and boundary review

- Reconciled `spec/NIP_DRAFT.md` SHA-256:
  `0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1`
- Reconciled `spec/NOSTR_AUTOMERGE_V1_SPEC.md` SHA-256:
  `58177c31eb06086d76297bbb0fc15343a8e34c15499d6e03636c63df7604bb10`
- The sealed revision, report schema, provisional kinds, tag contract, and
  standalone wire-format authority retain their pre-reconciliation identities.
- `NIP-XX`, `draft`, `optional`, provisional allocation, non-submission,
  nonpublication, nonrelease, and nonproduction status remain explicit.
- No upstream, remote, workflow, publication, or allocation action occurred.
