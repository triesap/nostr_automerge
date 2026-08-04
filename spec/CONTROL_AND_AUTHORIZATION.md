# Control and authorization contract

## Controller

The coordinate controller signs all controls and manifests.

The controller is a governance key. It is not an implicit writer.

## Genesis

A valid genesis:
- has sequence 0;
- has no parent;
- has empty base frontier;
- declares the sealed profile;
- contains at least one writer unless intentionally frozen;
- contains sorted unique devices and roles.

If multiple valid genesis events exist, apply the deterministic control
selection rule at the virtual root and surface controller equivocation.

## Child control

A child:
- has exactly one parent tag;
- sequence is parent + 1;
- carries complete device ACL;
- has sorted unique base_heads;
- satisfies role/device transition rules;
- base heads are accepted under parent chain;
- includes the retained writer history required by the NIP;
- is terminal/successor-consistent.

## Device transitions

- account mapping for an existing device cannot change;
- roles for an existing device can only decrease;
- a removed device never returns;
- privilege restoration/elevation uses a fresh device key;
- controller key may appear as a device only explicitly;
- no writer means frozen;
- terminal means no valid child.

## Base frontier

The child’s new epoch begins at exactly the ancestor closure of base_heads.

This causes intentional causal revocation:
- included parent work survives;
- concurrent parent work outside closure is excluded;
- new epoch work must descend from every base head;
- timestamps do not matter.

## Canonical selection

For each canonical parent:
- collect otherwise-valid child controls;
- select lowest decoded EventId;
- exclude sibling branches from canonical state;
- preserve evidence;
- rerun from genesis when new evidence can introduce a lower child.

## Alerts

Controller equivocation and reorganization are integrity alerts.

The library does not silently hide:
- candidate siblings;
- selected child;
- previous canonical tip;
- new canonical tip;
- affected changes.

## Authorization query

A change/checkpoint is authorized against the exact referenced canonical
control. Later role changes do not retroactively change valid earlier evidence;
frontier selection determines retained state.
