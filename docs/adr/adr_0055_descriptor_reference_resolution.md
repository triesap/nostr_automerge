# ADR 0055: Checkpoint Descriptor Reference Resolution

Status: Approved

## Context

A checkpoint chunk names a descriptor event. Absence from the valid descriptor
index cannot distinguish an undelivered descriptor from present wrong-kind,
wrong-coordinate, invalid, unsupported, or dynamically unusable evidence.

## Decision

Use one private descriptor-reference resolver over retained verified evidence
and dynamic checkpoint outcomes. It returns verified target descriptor,
pending, missing, wrong kind, wrong coordinate, statically invalid, dynamically
invalid, or unsupported.

Missing and pending descriptor evidence yields a pending chunk. Every present
known unusable descriptor reference makes a draft-v1 chunk invalid. A verified
descriptor proceeds through complete coordinate, descriptor ID, author, index,
count, size, proof, commitment, and snapshot binding before acceptance.

## Consequences

Every target chunk receives exactly one final event disposition. Genuine orphan
chunks can promote after descriptor delivery, while known invalid evidence
cannot masquerade as absence. Descriptor, chunk, checkpoint result, and digest
records remain mutually consistent.
