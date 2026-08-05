# ADR 0017: requirement coverage fails closed

## Decision

Every registered requirement has one closed applicability status and direct
implementation evidence where applicable. Mandatory missing or stale evidence
fails repository validation.

## Consequences

Reports cannot become green merely by listing missing requirements.
