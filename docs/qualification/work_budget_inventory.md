# Deterministic Work Inventory v3

This inventory covers every evidence-influenced traversal in the public batch
evaluator. A traversal is green only when it has a typed counter, a cooperative
cancellation boundary, or a sealed constant bound independent of retained
evidence.

| Area | Traversal owner | Required counter | Cancellation owner | Current state |
| --- | --- | --- | --- | --- |
| ingress summary | `EvidenceCorpus` event, carrier, and decode summaries | `event`, `carrier`, `decode_byte` | evaluator entry | metered |
| control preparation | `engine::reference_evaluator::prepare_controls` | `control` for collection, parent lookup, members, roles, account/device comparisons | before every control and proportional comparison charge | metered |
| control collection | `reference::evaluate::evaluate_batch` | `control` | each retained batch control before indexing | metered |
| child transition | `reference::evaluate::charge_control_transitions` and `control::candidate` | `control` before any candidate transition; graph counters for closures | epoch cancellation boundary | metered |
| control ancestry | `engine::reference_evaluator::build_control_ancestry_index` and incremental canonical ancestry in `reference::evaluate` | `control` per indexed node, memoized vector element, and comparison | every indexed node and epoch boundary | metered; one index plus one incremental canonical vector |
| frontier closure | `control::frontier::accepted_frontier_closure_metered` | exact `graph_node` and `graph_edge` visits for the iterative closure and antichain passes | immediately before each node, edge, comparison, and insertion | metered |
| actor reconstruction | `graph::actor_state::initialize_actor_states_metered` | exact `graph_node` and `graph_edge` visits for accepted-closure topology indexing and traversal | immediately before each closure/candidate/dependency operation | metered |
| dependency scheduling | `graph::schedule` and `graph::closure::candidate_dependency_closure` | `graph_node`, `graph_edge` | every queue, set, topology, and adjacency loop | metered |
| ancestor closure | `graph::closure::ancestor_closure` | `graph_node`, `graph_edge` | every stack and dependency loop | metered |
| equivocation grouping | `graph::equivocation` | `graph_node`, `graph_edge` | every candidate, carrier, group, affected-set, queue, descendant, and quarantine walk | metered |
| epoch fixpoint | `reference::epoch` | `graph_node`, `graph_edge`, `apply_change` | each pass and candidate | metered |
| Automerge decode | carrier qualification and adapter decode | `decode_byte` | before decode | metered |
| Automerge application | `reference::apply`, `automerge_adapter::document` | `apply_change`, `graph_edge` | each change and dependency | metered |
| checkpoint collection | `engine::reference_evaluator` trusted checkpoint indexes | `checkpoint_item` for canonical controls, controls, members, descriptors, and chunks | before every indexed preparation boundary | metered |
| checkpoint assembly | `checkpoint::join`, `checkpoint::assemble` | `checkpoint_item`, `checkpoint_byte` | each chunk and byte boundary | metered |
| checkpoint history | `engine::reference_evaluator` and `checkpoint::verify_history` | `checkpoint_item` for canonical-control coverage, refused-descriptor ancestry insertion and parent lookup, accepted snapshots, embedded changes, and membership checks | before every ordered-set insertion, parent lookup, history item, and set-membership check | metered; historical carriers use one same-or-ancestor set per refused descriptor and fail closed on missing/cyclic/wrong-coordinate ancestry |
| checkpoint load | `automerge_adapter::checkpoint` | `checkpoint_item`, `checkpoint_byte` | before load | metered |
| materialized projection | `automerge_adapter::materialized_view` | `decode_byte` and `apply_change` for snapshot load; `assertion` for objects, paths, properties, conflicts, values, text, marks, and bounded sorting | before every explicit stack, value, and sort boundary | metered |
| manifest resolution | `engine::reference_evaluator::resolve_selected_manifest` | `carrier` | immediately before selected-reference lookup | metered |
| dynamic event dispositions | `engine::reference_evaluator::event_disposition_records` | `carrier` proportional to retained event count | immediately before record construction | metered |
| final change lineage and carrier reduction | `engine::reference_evaluator::canonical_ancestor_hashes` and `reduce_change_dispositions` | `control` per canonical control/member and `graph_node`/`carrier` per accepted hash, semantic hash, and carrier Event | immediately before each indexed pull, membership classification, and direct outcome insertion | one charged lineage traversal plus fixed-size online aggregate state |
| evidence report collection | `EvidenceCorpus::records` | `event` proportional to retained evaluation evidence | immediately before collection | metered |
| canonical digests | history and disposition encoders | `assertion` proportional to exact canonical input count | immediately before digest input construction | metered |
| final report construction | `engine::reference_evaluator` | producing traversals plus the compact interruption path | every optional stage; no corpus scan after stop | metered or compact |
| checkpoint interruption | `engine::reference_evaluator::verify_checkpoints` | `checkpoint_item` before every result element | every descriptor, chunk identity, history vector, and head | bounded prefix; no refusal expansion |
| assertion matching | conformance assertion runner | `assertion` | each assertion and projected value | metered |

Every runtime row is closed. Test-only loops and loops bounded by a literal
protocol constant are outside runtime charging, but remain subject to the
source panic policy. Deep-control and many-checkpoint public-engine tests prove
deterministic work and compact interruption without recursive traversal or
post-stop refusal expansion.

The v10 operation inventory names twelve unique enabled tests, one per row.
The resource gate executes that exact proof set together with deep/wide graph
scaling, repeated accepted-state cache boundaries, many-checkpoint and
unrelated-evidence public cases, both finding reproductions, and the resource
benchmark. The inventory validator rejects source mutations that delete or
relocate charges, restore full copies or nested lineage scans, or reintroduce
sequence-based checkpoint history.

## Frozen Counter Semantics

`decode_byte` and `checkpoint_byte` consume byte capacity. Every other counter
consumes item capacity. A charge computes both the next counter and remaining
capacity before mutating either value, so overflow and exhaustion are atomic.
Cancellation is checked before the charge and before performing the associated
optional work. A failed charge never becomes protocol-invalid evidence.
