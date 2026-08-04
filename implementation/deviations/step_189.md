# Step 189 scope deviation: local-only workflow and readiness policy

Date: 2026-08-04
Status: approved; pending execution

## Prior plan

RCLD 13 and steps `step_189` through `step_192` were originally reserved for
editing and advancing a draft NIP through a separate NIPs checkout. RCLD 12
also treated committed GitHub Actions workflow definitions as acceptable
ongoing interoperability policy.

## Approved change

The NIP document, identifier allocation, event-kind allocation, upstream
issues, pull requests, and NIPs repository are outside this implementation
program. GitHub-hosted workflows are prohibited. Both implementation
repositories must instead use ignored, untracked `.act/workflows/**` runners
and prove all gates on the local machine.

RCLD 13 is therefore redefined as local implementation readiness. Its four
checkpoints will reconcile the runner policy, establish complete local lanes
for both repositories, prove independent local interoperability, and close
code-applicable specification coverage, robustness, resource, and optimization
evidence.

## Safety boundary

The revised work does not edit the NIP, a NIPs fork, registry data, remote
repositories, issues, pull requests, tags, releases, or packages. It does not
claim adoption, allocation, hosted CI, external review, relay compatibility,
or production qualification.

## Consequence

RCLD 12's protocol-agreement result remains valid, but its committed-workflow
policy and related readiness wording require correction during `step_189`.
The complete implementation program remains unfinished until revised RCLD 13
is green.
