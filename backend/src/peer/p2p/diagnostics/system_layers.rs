//! # System Layers - Diagnostik für alle Erynoa-Module
//!
//! Erweitert die P2P Layer-Diagnostik um System-weite Checks.

use super::system_state::SystemSnapshot;
use super::types::{DiagnosticCheck, LayerDiagnostic};

/// Generiert System-Layer-Diagnostik aus SystemSnapshot
pub fn generate_system_layers(snapshot: &SystemSnapshot) -> Vec<LayerDiagnostic> {
    vec![
        generate_trust_layer(snapshot),
        generate_event_layer(snapshot),
        generate_world_formula_layer(snapshot),
        generate_consensus_layer(snapshot),
        generate_eclvm_layer(snapshot),
        generate_mana_layer(snapshot),
        generate_policy_layer(snapshot),
        generate_storage_layer(snapshot),
        generate_archive_layer(snapshot),
        generate_anomaly_layer(snapshot),
        generate_diversity_layer(snapshot),
        generate_protection_layer(snapshot),
        generate_execution_layer(snapshot),
    ]
}

// ============================================================================
// CORE ENGINE LAYERS
// ============================================================================

/// L1: Trust Engine Layer (Κ2-Κ5)
fn generate_trust_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let t = &s.trust;
    let mut layer = LayerDiagnostic::new("Trust Engine (Κ2-Κ5)", 1)
        .with_description("Trust relationships and axiom enforcement");

    // Entities check
    let entity_check = if t.entities_count == 0 {
        DiagnosticCheck::degraded(
            "Registered Entities",
            format!("{} entities tracked", t.entities_count),
        )
    } else {
        DiagnosticCheck::healthy(
            "Registered Entities",
            format!("{} entities tracked", t.entities_count),
        )
    }
    .with_metric(t.entities_count as f64, "entities");
    layer.add_check(entity_check);

    // Trust value check (Κ3: 0 ≤ 𝕎 ≤ 1)
    let avg_check = if t.avg_trust_value < 0.3 {
        DiagnosticCheck::degraded(
            "Average Trust (Κ3)",
            format!("Κ3 bounds [0,1] - avg: {:.3}", t.avg_trust_value),
        )
    } else if t.avg_trust_value >= 0.0 && t.avg_trust_value <= 1.0 {
        DiagnosticCheck::healthy(
            "Average Trust (Κ3)",
            format!("Κ3 bounds satisfied - avg: {:.3}", t.avg_trust_value),
        )
    } else {
        DiagnosticCheck::unavailable(
            "Average Trust (Κ3)",
            format!("Κ3 bounds VIOLATED - avg: {:.3}", t.avg_trust_value),
        )
    }
    .with_metric(t.avg_trust_value, "trust");
    layer.add_check(avg_check);

    // Asymmetric Update check (Κ4: negative 2× faster)
    let ratio = if t.positive_updates > 0 {
        t.negative_updates as f64 / t.positive_updates as f64
    } else {
        0.0
    };
    let update_check = DiagnosticCheck::healthy(
        "Update Asymmetry (Κ4)",
        format!(
            "{}+ / {}- updates (ratio: {:.2})",
            t.positive_updates, t.negative_updates, ratio
        ),
    )
    .with_metric(ratio, "ratio");
    layer.add_check(update_check);

    // Trust violations
    let violation_check = if t.trust_violations > 10 {
        DiagnosticCheck::unavailable(
            "Trust Violations",
            format!("{} violations detected", t.trust_violations),
        )
    } else if t.trust_violations > 0 {
        DiagnosticCheck::degraded(
            "Trust Violations",
            format!("{} violations detected", t.trust_violations),
        )
    } else {
        DiagnosticCheck::healthy("Trust Violations", "No violations")
    }
    .with_metric(t.trust_violations as f64, "violations");
    layer.add_check(violation_check);

    layer
}

/// L2: Event Engine Layer (Κ9-Κ12)
fn generate_event_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let e = &s.events;
    let mut layer = LayerDiagnostic::new("Event Engine (Κ9-Κ12)", 2)
        .with_description("DAG management and finality tracking");

    // DAG integrity
    let dag_check = if e.cycles_detected > 0 {
        DiagnosticCheck::unavailable(
            "DAG Integrity (Κ9)",
            format!("{} CYCLES DETECTED!", e.cycles_detected),
        )
    } else {
        DiagnosticCheck::healthy(
            "DAG Integrity (Κ9)",
            format!("{} events, no cycles", e.events_total),
        )
    }
    .with_metric(e.events_total as f64, "events");
    layer.add_check(dag_check);

    // Finality (Κ10)
    let finality_ratio = if e.events_total > 0 {
        e.finalized_events as f64 / e.events_total as f64
    } else {
        1.0
    };
    let finality_check = if finality_ratio < 0.5 {
        DiagnosticCheck::degraded(
            "Finality (Κ10)",
            format!(
                "{:.1}% finalized ({}/{})",
                finality_ratio * 100.0,
                e.finalized_events,
                e.events_total
            ),
        )
    } else {
        DiagnosticCheck::healthy(
            "Finality (Κ10)",
            format!(
                "{:.1}% finalized ({}/{})",
                finality_ratio * 100.0,
                e.finalized_events,
                e.events_total
            ),
        )
    }
    .with_metric(finality_ratio * 100.0, "%");
    layer.add_check(finality_check);

    // Witnessing
    let witness_check = DiagnosticCheck::healthy(
        "Witnessing",
        format!("{} witnessed events", e.witnessed_events),
    )
    .with_metric(e.witnessed_events as f64, "events");
    layer.add_check(witness_check);

    // Validation errors
    let validation_check = if e.validation_errors > 20 {
        DiagnosticCheck::unavailable(
            "Validation",
            format!("{} validation errors", e.validation_errors),
        )
    } else if e.validation_errors > 5 {
        DiagnosticCheck::degraded(
            "Validation",
            format!("{} validation errors", e.validation_errors),
        )
    } else {
        DiagnosticCheck::healthy(
            "Validation",
            format!("{} validation errors", e.validation_errors),
        )
    }
    .with_metric(e.validation_errors as f64, "errors");
    layer.add_check(validation_check);

    layer
}

/// L3: World Formula Layer (Κ15b-d)
fn generate_world_formula_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let w = &s.world_formula;
    let mut layer = LayerDiagnostic::new("World Formula (Κ15b-d)", 3)
        .with_description("𝔼 = Σ 𝔸(s) · σ⃗(...) global state computation");

    // 𝔼 value
    let e_check = if w.current_e_value == 0.0 && w.contributors_count > 0 {
        DiagnosticCheck::degraded(
            "World State 𝔼 (Κ15b)",
            format!("𝔼 = {:.6}", w.current_e_value),
        )
    } else {
        DiagnosticCheck::healthy(
            "World State 𝔼 (Κ15b)",
            format!("𝔼 = {:.6}", w.current_e_value),
        )
    }
    .with_metric(w.current_e_value, "𝔼");
    layer.add_check(e_check);

    // Contributors
    let contrib_check = DiagnosticCheck::healthy(
        "Contributors",
        format!("{} active contributors", w.contributors_count),
    )
    .with_metric(w.contributors_count as f64, "contributors");
    layer.add_check(contrib_check);

    // Human verification (Ĥ factor)
    let human_ratio = if w.contributors_count > 0 {
        w.human_verified_count as f64 / w.contributors_count as f64
    } else {
        0.0
    };
    let human_check = DiagnosticCheck::healthy(
        "Human Factor Ĥ",
        format!(
            "{:.1}% verified ({}/{})",
            human_ratio * 100.0,
            w.human_verified_count,
            w.contributors_count
        ),
    )
    .with_metric(human_ratio * 100.0, "%");
    layer.add_check(human_check);

    // Computations
    let comp_check = DiagnosticCheck::healthy(
        "Computations",
        format!("{} total computations", w.computations_total),
    )
    .with_metric(w.computations_total as f64, "computations");
    layer.add_check(comp_check);

    layer
}

/// L4: Consensus Layer (Κ18)
fn generate_consensus_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let c = &s.consensus;
    let mut layer = LayerDiagnostic::new("Consensus (Κ18)", 4)
        .with_description("Epoch management and validator coordination");

    // Epoch
    let epoch_check =
        DiagnosticCheck::healthy("Current Epoch", format!("Epoch {}", c.current_epoch))
            .with_metric(c.current_epoch as f64, "epoch");
    layer.add_check(epoch_check);

    // Validators
    let validator_check = if c.validators_count < 3 {
        DiagnosticCheck::degraded(
            "Validators",
            format!(
                "{} active validators (min 3 recommended)",
                c.validators_count
            ),
        )
    } else {
        DiagnosticCheck::healthy(
            "Validators",
            format!("{} active validators", c.validators_count),
        )
    }
    .with_metric(c.validators_count as f64, "validators");
    layer.add_check(validator_check);

    // Success rate
    let total_rounds = c.successful_rounds + c.failed_rounds;
    let success_rate = if total_rounds > 0 {
        c.successful_rounds as f64 / total_rounds as f64
    } else {
        1.0
    };
    let rate_check = if success_rate < 0.9 {
        DiagnosticCheck::degraded(
            "Consensus Rate",
            format!(
                "{:.1}% success ({}/{})",
                success_rate * 100.0,
                c.successful_rounds,
                total_rounds
            ),
        )
    } else {
        DiagnosticCheck::healthy(
            "Consensus Rate",
            format!(
                "{:.1}% success ({}/{})",
                success_rate * 100.0,
                c.successful_rounds,
                total_rounds
            ),
        )
    }
    .with_metric(success_rate * 100.0, "%");
    layer.add_check(rate_check);

    layer
}

// ============================================================================
// ECLVM LAYERS
// ============================================================================

/// L5: ECLVM Runtime Layer
fn generate_eclvm_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let e = &s.eclvm;
    let mut layer = LayerDiagnostic::new("ECLVM Runtime", 5)
        .with_description("Virtual machine execution and gas metering");

    // Execution stats
    let success_rate = if e.programs_executed > 0 {
        e.successful_executions as f64 / e.programs_executed as f64
    } else {
        1.0
    };
    let exec_check = if success_rate < 0.8 {
        DiagnosticCheck::degraded(
            "Execution Success",
            format!(
                "{:.1}% success ({}/{})",
                success_rate * 100.0,
                e.successful_executions,
                e.programs_executed
            ),
        )
    } else {
        DiagnosticCheck::healthy(
            "Execution Success",
            format!(
                "{:.1}% success ({}/{})",
                success_rate * 100.0,
                e.successful_executions,
                e.programs_executed
            ),
        )
    }
    .with_metric(success_rate * 100.0, "%");
    layer.add_check(exec_check);

    // Gas consumption
    let gas_check = DiagnosticCheck::healthy(
        "Gas Consumed",
        format!(
            "{} total ({:.0} avg/program)",
            e.total_gas_consumed, e.avg_gas_per_program
        ),
    )
    .with_metric(e.total_gas_consumed as f64, "gas");
    layer.add_check(gas_check);

    // Out of gas
    let oog_check = if e.out_of_gas_count > 10 {
        DiagnosticCheck::degraded("Out of Gas", format!("{} OOG events", e.out_of_gas_count))
    } else {
        DiagnosticCheck::healthy("Out of Gas", format!("{} OOG events", e.out_of_gas_count))
    }
    .with_metric(e.out_of_gas_count as f64, "OOG");
    layer.add_check(oog_check);

    // Active VMs
    let vm_check = DiagnosticCheck::healthy("Active VMs", format!("{} running", e.active_vms))
        .with_metric(e.active_vms as f64, "VMs");
    layer.add_check(vm_check);

    layer
}

/// L6: Mana System Layer
fn generate_mana_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let m = &s.mana;
    let mut layer = LayerDiagnostic::new("Mana System", 6)
        .with_description("Bandwidth-based reputation and rate limiting");

    // Accounts
    let acc_check = DiagnosticCheck::healthy(
        "Mana Accounts",
        format!("{} registered accounts", m.accounts_count),
    )
    .with_metric(m.accounts_count as f64, "accounts");
    layer.add_check(acc_check);

    // Flow
    let flow_check = DiagnosticCheck::healthy(
        "Mana Flow",
        format!(
            "{} consumed / {} regenerated",
            m.total_mana_consumed, m.total_mana_regenerated
        ),
    )
    .with_metric(m.total_mana_consumed as f64, "mana");
    layer.add_check(flow_check);

    // Rate limiting
    let rl_check = if m.rate_limited_requests > 100 {
        DiagnosticCheck::degraded(
            "Rate Limited",
            format!("{} requests limited", m.rate_limited_requests),
        )
    } else {
        DiagnosticCheck::healthy(
            "Rate Limited",
            format!("{} requests limited", m.rate_limited_requests),
        )
    }
    .with_metric(m.rate_limited_requests as f64, "limited");
    layer.add_check(rl_check);

    layer
}

/// L7: Policy Gateway Layer
fn generate_policy_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let p = &s.policies;
    let mut layer = LayerDiagnostic::new("Policy Gateway", 7)
        .with_description("Programmable access control and policy evaluation");

    // Policies
    let pol_check = DiagnosticCheck::healthy(
        "Registered Policies",
        format!("{} active policies", p.policies_count),
    )
    .with_metric(p.policies_count as f64, "policies");
    layer.add_check(pol_check);

    // Evaluations
    let allow_rate = if p.evaluations_total > 0 {
        p.allowed_requests as f64 / p.evaluations_total as f64
    } else {
        1.0
    };
    let eval_check = DiagnosticCheck::healthy(
        "Allow Rate",
        format!(
            "{:.1}% allowed ({}/{})",
            allow_rate * 100.0,
            p.allowed_requests,
            p.evaluations_total
        ),
    )
    .with_metric(allow_rate * 100.0, "%");
    layer.add_check(eval_check);

    // Denied
    let denied_check =
        DiagnosticCheck::healthy("Denied Requests", format!("{} denied", p.denied_requests))
            .with_metric(p.denied_requests as f64, "denied");
    layer.add_check(denied_check);

    layer
}

// ============================================================================
// STORAGE LAYERS
// ============================================================================

/// L8: Local Storage Layer
fn generate_storage_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let st = &s.storage;
    let mut layer = LayerDiagnostic::new("Local Storage", 8)
        .with_description("Persistent storage for events, identities, and content");

    // KV Store
    let kv_check = DiagnosticCheck::healthy(
        "KV Store",
        format!(
            "{} keys, {}",
            st.kv_store_keys,
            format_bytes(st.kv_store_bytes)
        ),
    )
    .with_metric(st.kv_store_bytes as f64, "bytes");
    layer.add_check(kv_check);

    // Event Store
    let event_check = DiagnosticCheck::healthy(
        "Event Store",
        format!("{} events stored", st.event_store_events),
    )
    .with_metric(st.event_store_events as f64, "events");
    layer.add_check(event_check);

    // Identity Store
    let id_check = DiagnosticCheck::healthy(
        "Identity Store",
        format!("{} identities", st.identity_store_entries),
    )
    .with_metric(st.identity_store_entries as f64, "identities");
    layer.add_check(id_check);

    // Realm Storage
    let realm_check =
        DiagnosticCheck::healthy("Realms", format!("{} active realms", st.realm_count))
            .with_metric(st.realm_count as f64, "realms");
    layer.add_check(realm_check);

    layer
}

/// L9: Archive Layer
fn generate_archive_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let a = &s.archive;
    let mut layer = LayerDiagnostic::new("Cold Archive (ψ_archive)", 9)
        .with_description("Merkle-tree based epoch archival");

    // Epochs
    let epoch_check = DiagnosticCheck::healthy(
        "Archived Epochs",
        format!("{} epochs finalized", a.archived_epochs),
    )
    .with_metric(a.archived_epochs as f64, "epochs");
    layer.add_check(epoch_check);

    // Events
    let event_check = DiagnosticCheck::healthy(
        "Archived Events",
        format!("{} events archived", a.archived_events),
    )
    .with_metric(a.archived_events as f64, "events");
    layer.add_check(event_check);

    // Merkle Roots
    let merkle_check = DiagnosticCheck::healthy(
        "Merkle Roots",
        format!("{} roots preserved", a.merkle_roots_count),
    )
    .with_metric(a.merkle_roots_count as f64, "roots");
    layer.add_check(merkle_check);

    // Size
    let size_check = DiagnosticCheck::healthy("Archive Size", format_bytes(a.archive_size_bytes))
        .with_metric(a.archive_size_bytes as f64, "bytes");
    layer.add_check(size_check);

    layer
}

// ============================================================================
// PROTECTION LAYERS
// ============================================================================

/// L10: Anomaly Detection Layer
fn generate_anomaly_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let a = &s.anomaly;
    let mut layer = LayerDiagnostic::new("Anomaly Detection", 10)
        .with_description("Velocity, amount, and pattern anomaly detection");

    // Critical anomalies
    let critical = a
        .anomalies_by_severity
        .get("critical")
        .copied()
        .unwrap_or(0);
    let critical_check = if critical > 0 {
        DiagnosticCheck::unavailable(
            "Critical Anomalies",
            format!("{} CRITICAL detected!", critical),
        )
    } else {
        DiagnosticCheck::healthy("Critical Anomalies", "No critical anomalies")
    }
    .with_metric(critical as f64, "critical");
    layer.add_check(critical_check);

    // High anomalies
    let high = a.anomalies_by_severity.get("high").copied().unwrap_or(0);
    let high_check = if high > 5 {
        DiagnosticCheck::degraded("High Anomalies", format!("{} high severity", high))
    } else {
        DiagnosticCheck::healthy("High Anomalies", format!("{} high severity", high))
    }
    .with_metric(high as f64, "high");
    layer.add_check(high_check);

    // Total
    let total_check = DiagnosticCheck::healthy(
        "Total Anomalies",
        format!("{} total detected", a.anomalies_detected),
    )
    .with_metric(a.anomalies_detected as f64, "anomalies");
    layer.add_check(total_check);

    layer
}

/// L11: Diversity Monitor Layer (Κ20)
fn generate_diversity_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let d = &s.diversity;
    let mut layer = LayerDiagnostic::new("Diversity Monitor (Κ20)", 11)
        .with_description("Shannon entropy tracking across dimensions");

    // Entropy threshold (Κ20)
    let entropy_check = if d.threshold_met {
        DiagnosticCheck::healthy(
            "Entropy Threshold (Κ20)",
            format!("Min entropy: {:.2} (OK)", d.min_entropy_current),
        )
    } else {
        DiagnosticCheck::degraded(
            "Entropy Threshold (Κ20)",
            format!(
                "Min entropy: {:.2} (BELOW THRESHOLD)",
                d.min_entropy_current
            ),
        )
    }
    .with_metric(d.min_entropy_current, "entropy");
    layer.add_check(entropy_check);

    // Monitored dimensions
    let dim_check = DiagnosticCheck::healthy(
        "Monitored Dimensions",
        format!("{} dimensions tracked", d.monitored_dimensions),
    )
    .with_metric(d.monitored_dimensions as f64, "dimensions");
    layer.add_check(dim_check);

    // Monoculture warnings
    let mono_check = if d.monoculture_warnings > 0 {
        DiagnosticCheck::degraded(
            "Monoculture Warnings",
            format!("{} warnings", d.monoculture_warnings),
        )
    } else {
        DiagnosticCheck::healthy("Monoculture Warnings", "No monoculture detected")
    }
    .with_metric(d.monoculture_warnings as f64, "warnings");
    layer.add_check(mono_check);

    layer
}

/// L12: Protection Systems (Κ19, Κ21)
fn generate_protection_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let ac = &s.anti_calcification;
    let q = &s.quadratic;
    let mut layer = LayerDiagnostic::new("Protection (Κ19, Κ21)", 12)
        .with_description("Anti-calcification and quadratic governance");

    // Anti-Calcification (Κ19)
    let ac_check = if ac.threshold_violations > 0 {
        DiagnosticCheck::degraded(
            "Anti-Calcification (Κ19)",
            format!(
                "{} interventions, {} violations, {} watched",
                ac.interventions_count, ac.threshold_violations, ac.entities_under_watch
            ),
        )
    } else {
        DiagnosticCheck::healthy(
            "Anti-Calcification (Κ19)",
            format!(
                "{} interventions, {} watched entities",
                ac.interventions_count, ac.entities_under_watch
            ),
        )
    }
    .with_metric(ac.interventions_count as f64, "interventions");
    layer.add_check(ac_check);

    // Power concentration
    let power_check = DiagnosticCheck::healthy(
        "Power Concentration",
        format!(
            "Index: {:.3}, Gini: {:.3}",
            ac.power_concentration_index, ac.gini_coefficient
        ),
    )
    .with_metric(ac.power_concentration_index, "index");
    layer.add_check(power_check);

    // Quadratic Governance (Κ21)
    let quad_check = DiagnosticCheck::healthy(
        "Quadratic Voting (Κ21)",
        format!(
            "{} active, {} completed, {} participants",
            q.active_votes, q.completed_votes, q.total_participants
        ),
    )
    .with_metric(q.completed_votes as f64, "votes");
    layer.add_check(quad_check);

    layer
}

// ============================================================================
// EXECUTION LAYER
// ============================================================================

/// L13: Execution Context Layer
fn generate_execution_layer(s: &SystemSnapshot) -> LayerDiagnostic {
    let e = &s.execution;
    let mut layer = LayerDiagnostic::new("Execution Context", 13)
        .with_description("IPS monad execution with gas accounting");

    // Active contexts
    let ctx_check =
        DiagnosticCheck::healthy("Active Contexts", format!("{} running", e.active_contexts))
            .with_metric(e.active_contexts as f64, "contexts");
    layer.add_check(ctx_check);

    // Success rate
    let total = e.successful_executions + e.failed_executions;
    let success_rate = if total > 0 {
        e.successful_executions as f64 / total as f64
    } else {
        1.0
    };
    let exec_check = if success_rate < 0.9 {
        DiagnosticCheck::degraded(
            "Execution Success",
            format!(
                "{:.1}% ({}/{})",
                success_rate * 100.0,
                e.successful_executions,
                total
            ),
        )
    } else {
        DiagnosticCheck::healthy(
            "Execution Success",
            format!(
                "{:.1}% ({}/{})",
                success_rate * 100.0,
                e.successful_executions,
                total
            ),
        )
    }
    .with_metric(success_rate * 100.0, "%");
    layer.add_check(exec_check);

    // Gas & Events
    let gas_check = DiagnosticCheck::healthy(
        "Gas & Events",
        format!(
            "{} gas, {} events emitted",
            e.total_gas_consumed, e.events_emitted
        ),
    )
    .with_metric(e.total_gas_consumed as f64, "gas");
    layer.add_check(gas_check);

    // Lamport clock
    let clock_check = DiagnosticCheck::healthy(
        "Clock State",
        format!("Epoch {}, Lamport {}", e.current_epoch, e.current_lamport),
    )
    .with_metric(e.current_lamport as f64, "lamport");
    layer.add_check(clock_check);

    layer
}

// ============================================================================
// HELPERS
// ============================================================================

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
