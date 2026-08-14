# ADR 0053: Dependent Carrier Control Mapping

Status: Approved

## Context

A dependent draft-v1 carrier has its own known revision and validity. The state
of a referenced control is evidence used to authorize that carrier; it is not a
revision label that can be inherited. Branch lineage also cannot excuse a
failed device, role, actor, or terminal-control authorization check.

## Decision

Resolve the referenced control, map its state for the dependent carrier, enforce
terminal and ACL authorization, and only then apply canonical versus valid
noncanonical lineage. Missing or pending references remain pending. Wrong-kind,
wrong-coordinate, statically invalid, dynamically invalid, and unsupported
references make a known-v1 dependent carrier invalid.

For changes, the device must hold the write role at the referenced control and
the derived ActorId must match the semantic change actor. A terminal control has
no writer and cannot authorize a change. Only an otherwise-valid authorized
claim under a valid noncanonical control is excluded.

## Consequences

Unsupported evidence remains reportable as unsupported in its own event
namespace, but cannot turn a known-v1 dependent carrier into unsupported.
Consumer mappings are exhaustive, independently implemented, fixture-backed,
and covered by stable diagnostic and mutation assertions.
