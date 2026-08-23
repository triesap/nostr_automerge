# Exact Evaluation Resource Inventory

This inventory binds every target-proportional reference-evaluation operation
to a deterministic work counter or to an atomically reserved finalization
dimension. Corpus index construction happens once at corpus finalization and is
outside an individual evaluation.

| Phase | Operation | Accounting |
| --- | --- | --- |
| Entry | cancellation before coordinate lookup | constant, checked first |
| Entry | borrow coordinate index and scalar metadata | constant |
| Ingress | target input events and carrier evidence | `event`, `carrier` |
| Controls | candidates, relationships, ACL members | `control`, `graph_node` |
| Prior knowledge | selected controls and target hashes | `control`, `graph_node` |
| Prior knowledge | carrier claims and referenced controls | `carrier`, `control` |
| Prior knowledge | ACL member and role comparisons | `control` |
| Changes | graph nodes, edges, decoding, application | typed graph, decode, and apply counters |
| Manifests | indexed target candidates | `carrier` |
| Checkpoints | descriptors, chunks, bytes, history | `checkpoint_item`, `checkpoint_byte` |
| Projection | values, text units, paths, bytes | typed projection counters |
| Finalization | control vectors and records | reserved `controls` |
| Finalization | change dispositions and derived sets | reserved `changes` |
| Finalization | event records and overlays | reserved `events` |
| Finalization | checkpoint results and chunk records | reserved `checkpoints` |
| Finalization | digest items and hashes | reserved `digest_items` |
| Finalization | evidence records and duplicates | reserved `evidence_records` |
| Finalization | report invariants | reserved `invariant_items` |
| Finalization | constant report construction | reserved `fixed_overhead` |

Evaluation must not scan another coordinate. Cancellation and exhaustion stop
prior-knowledge classification without inspecting later inputs. Reservation is
atomic, cross-dimension borrowing is forbidden, and a finalization permit may
finish only with a zero remainder. The complete path validates the report
before refunding demonstrably unused optional capacity.

Complete-report reservations account for every owned list construction and
digest pass. The invariant reservation is a checked conservative function of
target controls, hashes, Events, evidence, relationships, and checkpoint
records. Materialized-document authority reuses a digest computed during the
metered canonical-byte projection rather than rereading snapshot bytes during
report validation.
