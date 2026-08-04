# Specification provenance

The initial protocol authority was imported from the approved
`nostr_automerge_v1_spec_handoff_v1` implementation-baseline artifact generated
on 2026-08-04.

The SHA-256 of its canonical `package_manifest.json` is:

```text
e0777e8330d60c8595b8ba2f6c3fb86cdbae91fe6217d85daa4eaf40ed84e408
```

A copy of that source manifest is retained at
`docs/provenance/source_package_manifest.json`. The complete import and
adaptation mapping is `docs/import_adaptation.json`.

## Import policy

- Protocol and wire content is copied without semantic change.
- Repository ownership is adapted from `radrootslabs/nostr_automerge` to
  `triesap/nostr_automerge`.
- Requirement source paths are adapted from the source artifact's `specs/`
  layout to this repository's `spec/` layout.
- Every adapted file records both source and repository SHA-256 values.
- Later consensus-affecting changes follow the repository change-control and
  deviation policies introduced by the implementation sequence.

This provenance names a portable source artifact. It does not depend on a
private filesystem location, checkout layout, or prior conversation.
