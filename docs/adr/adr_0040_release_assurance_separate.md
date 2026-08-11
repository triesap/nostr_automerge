# ADR 0040: Release Assurance Remains Separate

Status: Approved

## Context

Local implementation completion cannot establish sustained fuzzing,
independent external review, or publication authority.

## Decision

Sustained native fuzzing and independent review remain separate publication
holds until they actually occur. No remediation checkpoint authorizes a
publication action.

## Consequences

Evidence reports implementation, fuzzing, independent review, and publication
authority independently. The strongest authorized terminal state is
`code_complete_publication_held`.
