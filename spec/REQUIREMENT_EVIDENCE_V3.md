# Requirement Evidence V3

Requirement coverage is an executed-evidence claim, not a source-path claim.
Each covered row binds an approved applicability classification, an exact
implementation identity and commit, an implementation path, an exact Cargo
test or signed fixture identifier, the command and job that executed it, and a
passing result artifact with its SHA-256 digest. Signed-fixture proofs also
bind the top-level fixture-distribution digest.

Applicability is maintained as reviewed authority data. A generator may not
infer or default a requirement to out-of-core or deferred. Requirements that
need both implementations remain held until independently produced TypeScript
evidence satisfies the same contract. Held evidence is never reported as a
pass.

The validator rejects missing or duplicate rows, unknown fields, stale
authority, absent result artifacts, non-passing results, nonexistent executed
identifiers, hash drift, cross-implementation substitution, and unapproved
applicability. Prior coverage formats remain historical and cannot satisfy the
current repository gate.
