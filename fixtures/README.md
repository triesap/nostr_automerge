# Fixtures

This directory defines language-neutral inputs and expected reports.

The included actor derivation fixture is an illustrative seed. History and
disposition digest fields are zero placeholders because that fixture does not
evaluate document history. A production fixture generator must use the
approved digest contract for document scenarios.

Malformed protocol fixtures must preserve raw bytes/files rather than only
parsed JSON.

Every fixture input and expected report is covered by its fixture metadata
SHA-256 values and repository validation.
