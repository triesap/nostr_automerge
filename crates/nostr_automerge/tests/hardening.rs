//! Hardening configuration contract tests.
#[test]
fn no_untrusted_panic_paths() {
    fn rust_sources(directory: &std::path::Path, output: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                rust_sources(&path, output);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                output.push(path);
            }
        }
    }

    let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    rust_sources(&source_root, &mut sources);
    assert!(!sources.is_empty());
    for path in sources {
        let source = std::fs::read_to_string(&path);
        assert!(source.is_ok(), "failed to read {}", path.display());
        let Ok(source) = source else { return };
        let test_boundaries = [
            "#[cfg(test)]\nmod tests",
            "#[cfg(test)]\npub(crate) mod tests",
        ];
        let production_end = test_boundaries
            .iter()
            .filter_map(|boundary| source.find(boundary))
            .min()
            .unwrap_or(source.len());
        let production = &source[..production_end];
        for forbidden in ["unreachable!", "panic!", ".unwrap()", ".expect("] {
            assert!(
                !production.contains(forbidden),
                "{} contains production panic path {forbidden}",
                path.display()
            );
        }
    }
}

#[test]
fn projection_v2_vectors_cover_the_complete_neutral_model() {
    let vectors: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/projection/v2_vectors.json"
    ))
    .unwrap_or_default();
    assert_eq!(vectors["schema"], "nostr_automerge.projection_vectors.v2");
    assert_eq!(vectors["requirements"].as_array().map(Vec::len), Some(4));
    assert_eq!(vectors["scalar_types"].as_array().map(Vec::len), Some(9));
    assert_eq!(vectors["object_types"].as_array().map(Vec::len), Some(4));
    assert_eq!(vectors["mark_expansions"].as_array().map(Vec::len), Some(4));
    assert_eq!(
        vectors["conflicting_descendant"]["branch_qualified_paths"],
        2
    );
    assert_eq!(vectors["text"]["utf16_length"], 5);
    assert_eq!(vectors["deep_iterative_depth"], 2048);
    assert_eq!(vectors["canonical"], true);
}

#[test]
fn initialize_cargo_fuzz_harness() {
    let manifest = include_str!("../../../fuzz/Cargo.toml");
    assert!(manifest.contains("cargo-fuzz"));
    assert!(manifest.contains("name = \"smoke\""));
}
#[test]
fn fuzz_strict_raw_json_and_nip_01() {
    assert!(
        include_str!("../../../fuzz/fuzz_targets/raw_nip01.rs")
            .contains("VerifiedNip01Event::verify")
    );
}
#[test]
fn fuzz_automerge_framing_and_semantic_decode() {
    assert!(include_str!("../../../fuzz/fuzz_targets/automerge_framing.rs").contains("framing"));
    assert!(include_str!("../../../fuzz/fuzz_targets/automerge_decode.rs").contains("decode"));
    assert!(include_str!("../../../fuzz/fuzz_targets/automerge_reencode.rs").contains("reencode"));
}
#[test]
fn fuzz_materialized_projection() {
    assert!(
        include_str!("../../../fuzz/fuzz_targets/projection.rs")
            .contains("qualification_probe_projection")
    );
}
#[test]
fn fuzz_control_objects_and_transitions() {
    assert!(
        include_str!("../../../fuzz/fuzz_targets/control_transition.rs")
            .contains("qualification_probe_control")
    );
}
#[test]
fn fuzz_dependency_graph_and_evaluator() {
    assert!(
        include_str!("../../../fuzz/fuzz_targets/reference_evaluator.rs")
            .contains("qualification_probe_reference")
    );
}
#[test]
fn fuzz_checkpoint_parser_and_merkle_verification() {
    assert!(include_str!("../../../fuzz/fuzz_targets/checkpoint.rs").contains("merkle_root"));
}
#[test]
fn add_resource_and_performance_benchmarks() {
    let report: serde_json::Value =
        serde_json::from_str(include_str!("../../../reports/resource_benchmarks.json"))
            .unwrap_or_default();
    assert_eq!(report["schema"], "nostr_automerge.resource_benchmarks.v3");
    assert_eq!(report["ceilings"]["checkpoint_smoke_leaves"], 4096);
    assert_eq!(report["graph_models"]["chain_128"]["graph_nodes"], 256);
    assert_eq!(report["graph_models"]["fan_in_128"]["graph_edges"], 254);
    assert_eq!(report["rust"]["work_counter_boundary_status"], "pass");
    assert_eq!(report["result"], "pass");
}
#[test]
fn add_mutation_tests_for_critical_validators() {
    let config = include_str!("../../../mutants.toml");
    assert!(config.contains("wire,control,graph,checkpoint"));
}
#[test]
fn add_coverage_reporting() {
    let policy = include_str!("../../../docs/coverage.md");
    assert!(policy.contains("cargo +nightly-2026-07-16 llvm-cov --branch"));
    assert!(policy.contains("--exclude nostr_automerge_xtask --locked"));
    assert!(policy.contains("external local runner"));
}
#[test]
fn add_dependency_advisory_and_license_policy() {
    let policy = include_str!("../../../deny.toml");
    assert!(policy.contains("unknown-git = \"deny\""));
    assert!(include_str!("../../../Cargo.toml").contains("automerge = { version = \"=0.10.0\""));
}
#[test]
fn complete_public_documentation_and_examples() {
    let readme = include_str!("../../../README.md");
    assert!(readme.contains("Validation and checkpoints"));
    assert!(readme.contains("full replay remains required"));
}
#[test]
fn review_public_api_and_semver_surface() {
    let report = include_str!("../../../reports/api_review.md");
    for forbidden in ["pub automerge::", "pub secp256k1::", "pub serde_json::"] {
        assert!(!report.contains(forbidden));
    }
    assert!(include_str!("../../../reports/public_api.txt").contains("third_party_types: none"));
}
#[test]
fn prepare_alpha_package_and_clean_checkout_verificat() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("version.workspace = true"));
    assert!(manifest.contains("include = ["));
    assert!(include_str!("../../../CHANGELOG.md").contains("0.1.0-alpha.0"));
}
#[test]
fn publish_security_and_release_readiness_report() {
    let report: serde_json::Value =
        serde_json::from_str(include_str!("../../../reports/release_readiness.json"))
            .unwrap_or_default();
    assert_eq!(report["decision"], "hold_publication");
    assert_eq!(
        report["local_alpha_package"],
        "artifact_and_reproducibility_verified"
    );
    assert_eq!(
        report["public_engine"],
        "substantial_alpha_remediation_required"
    );
    assert_eq!(report["code_completion"], "follow_up_remediation_required");
    assert_eq!(report["external_review"], "not_completed_release_hold");
    assert_eq!(report["locked_gate"], "pass");
}

#[test]
fn publish_finding_by_finding_remediation_closure() {
    let report: serde_json::Value =
        serde_json::from_str(include_str!("../../../reports/remediation_closure.json"))
            .unwrap_or_default();
    assert_eq!(report["findings"].as_array().map(Vec::len), Some(13));
    let results = report["findings"]
        .as_array()
        .map(|findings| {
            findings
                .iter()
                .map(|finding| finding["result"].as_str().unwrap_or_default())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(results[..11], ["closed"; 11]);
    assert_eq!(results[11], "closed_locally_with_release_holds");
    assert_eq!(results[12], "closed");
    assert_eq!(
        report["status"],
        "historical_step_307_claim_superseded_by_follow_up_remediation"
    );
    assert!(report["non_claims"].as_array().is_some_and(|claims| {
        claims
            .iter()
            .any(|claim| claim == "no sustained native Rust fuzz execution")
            && claims
                .iter()
                .any(|claim| claim == "no independent external security or protocol review")
    }));
}

#[test]
fn close_local_implementation_scope_without_release_overclaim() {
    let report: serde_json::Value =
        serde_json::from_str(include_str!("../../../reports/implementation_scope.json"))
            .unwrap_or_default();
    assert_eq!(report["checkpoint_range"], "step_000_through_step_307");
    assert_eq!(report["code_scope"]["requirements_classified"], 87);
    assert_eq!(report["code_scope"]["findings_closed"], 12);
    assert_eq!(report["code_scope"]["findings_closed_with_release_hold"], 1);
    assert_eq!(report["nip_document"], "out_of_scope_not_modified");
    assert_eq!(
        report["status"],
        "historical_step_307_claim_superseded_by_follow_up_remediation"
    );
    assert_eq!(report["superseded_by"], "rcld_15_through_rcld_28");
    assert_eq!(report["release_holds"].as_array().map(Vec::len), Some(3));
}
