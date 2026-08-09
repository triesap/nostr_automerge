# Draft V1 Remediation Findings

All findings were open at the `step_193` baseline. Findings 001 through 011 and
013 are closed by executable regression evidence and their owning phase's
end-to-end proof. Finding 012 is closed by a fail-closed publication hold:
sustained Rust fuzz execution and independent external review remain expressly
unproven, so no production or publication readiness is claimed.

| Finding | Severity | Owning phase |
| --- | --- | --- |
| `FINDING_001` | blocker | public engine API |
| `FINDING_002` | blocker | public engine API |
| `FINDING_003` | critical | evaluator correctness |
| `FINDING_004` | high | graph hardening |
| `FINDING_005` | high | graph hardening |
| `FINDING_006` | blocker | checkpoint carrier integration |
| `FINDING_007` | blocker | state projection and conformance |
| `FINDING_008` | high | coverage, interop, and release |
| `FINDING_009` | high | checkpoint carrier integration |
| `FINDING_010` | blocker | state projection and conformance |
| `FINDING_011` | medium | control alignment |
| `FINDING_012` | high | coverage, interop, and release |
| `FINDING_013` | high | coverage, interop, and release |

The machine registry is authoritative for exact titles, evidence paths,
baseline identity, status, and phase identifiers.
