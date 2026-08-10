# Deterministic Work Inventory

This inventory covers every evidence-influenced traversal in the public batch
evaluator. A traversal is green only when it has a typed counter, a cooperative
cancellation boundary, or a sealed constant bound independent of retained
evidence.

| Area | Traversal owner | Required counter | Cancellation owner | Current state |
| --- | --- | --- | --- | --- |
| ingress summary | `EvidenceCorpus` event, carrier, and decode summaries | `event`, `carrier`, `decode_byte` | evaluator entry | metered |
| control collection | `reference::evaluate::evaluate_batch` | `control` | each retained batch control before indexing | metered |
| child transition | `reference::evaluate::charge_control_transitions` and `control::candidate` | `control` before any candidate transition; graph counters for closures | epoch cancellation boundary | metered |
| control ancestry | `reference::evaluate::collect_control_ancestry` and checkpoint ancestry lookup | `control` per canonical ancestor | before every lookup | metered for evaluation; checkpoint path pending |
| frontier closure | `reference::evaluate::charge_control_closures` and `control::frontier` | conservative `graph_node` and `graph_edge` precharge for every closure pass | before nodes and before edges | metered |
| actor reconstruction | `reference::epoch_engine::charge_actor_reconstruction` and `graph::actor_state` | conservative `graph_node` and `graph_edge` precharge for topology indexing and traversal | before reconstruction and between node/edge charges | metered |
| dependency scheduling | `graph::schedule` | `graph_node`, `graph_edge` | every queue and adjacency loop | metered |
| ancestor closure | `graph::closure::ancestor_closure` | `graph_node`, `graph_edge` | every stack and dependency loop | metered |
| equivocation grouping | `graph::equivocation` | `graph_node`, `graph_edge` | grouping and quarantine walks | partial |
| epoch fixpoint | `reference::epoch` | `graph_node`, `graph_edge`, `apply_change` | each pass and candidate | partial |
| Automerge decode | carrier qualification and adapter decode | `decode_byte` | before decode | metered |
| Automerge application | `reference::apply`, `automerge_adapter::document` | `apply_change`, `graph_edge` | each change and dependency | partial |
| checkpoint collection | `engine::reference_evaluator` | `checkpoint_item`, `checkpoint_byte` | each descriptor and chunk | remediation required |
| checkpoint assembly | `checkpoint::join`, `checkpoint::assemble` | `checkpoint_item`, `checkpoint_byte` | each chunk and byte boundary | partial |
| checkpoint history | `checkpoint::verify`, `checkpoint::verify_history` | `checkpoint_item`, `checkpoint_byte` | graph and carrier walks | remediation required |
| checkpoint load | `automerge_adapter::checkpoint` | `checkpoint_item`, `checkpoint_byte` | before load | metered |
| materialized projection | `automerge_adapter::materialized_view` | `assertion`, `apply_change` | object, property, conflict, text, and mark loops | remediation required |
| assertion matching | conformance assertion runner | `assertion` | each assertion and projected value | remediation required |
| canonical digests | history and disposition encoders | sealed report-size input already charged by producing traversal | caller boundary | derived-only |

The implementation checkpoints following this inventory close every row marked
`partial` or `remediation required`. Test-only loops and loops bounded by a
literal protocol constant are outside runtime charging, but remain subject to
the source panic policy.

## Frozen Counter Semantics

`decode_byte` and `checkpoint_byte` consume byte capacity. Every other counter
consumes item capacity. A charge computes both the next counter and remaining
capacity before mutating either value, so overflow and exhaustion are atomic.
Cancellation is checked before the charge and before performing the associated
optional work. A failed charge never becomes protocol-invalid evidence.
