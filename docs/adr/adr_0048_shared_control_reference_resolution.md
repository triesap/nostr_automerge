# ADR 0048: Shared Control-Reference Resolution

Status: Approved

## Context

Manifests, change claims, dependencies, and checkpoints currently infer the
meaning of referenced control evidence through separate lookup paths. Map
absence can therefore collapse missing, pending, invalid, unsupported, and
noncanonical controls into the wrong outcome.

## Decision

Introduce one private resolver with exhaustive canonical,
statefully-valid-noncanonical, pending, missing, wrong-kind, wrong-coordinate,
statically-invalid, dynamically-invalid, and unsupported states. It derives
state only from retained signed evidence plus stateful control outcomes.

Each consumer applies its own role rule after resolution. A dependent draft-v1
carrier that references unsupported control evidence is invalid; it does not
inherit the referenced event's unsupported revision. Stable diagnostics remain
outside canonical digest identity.

## Consequences

All dependent carrier consumers share one deterministic, synchronous,
network-free resolution boundary. No third-party protocol type enters the public
API, and exhaustive unit, signed, permutation, and mutation tests cover every
state and consumer mapping.
