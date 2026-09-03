# Causal projection v16 ledger

`step_1469` opens RCLD 125 at reviewed predecessor
`1d44643af3031de52cc0bc398f06f9174b846ab9`. It installs the v16 governing
plan, authority, findings, baseline, runtime cursor, and validation routes
without modifying production behavior. Findings 116 through 118 are open.
Finding 080 and all external-action holds remain held. The final operation
count remains undiscovered. The next checkpoint is `step_1470`.

`step_1470` adds four exact ignored Rust reproductions for Finding 116. They
prove that actor classification remains outside an owned stage, an actor
failure can reach the eager causal start comparison, the start counter is
compared twice, and budget or cancellation can stop at that premature work.
The report is expected-defect evidence and does not claim closure. Production
behavior is unchanged. The next checkpoint is `step_1471`.
