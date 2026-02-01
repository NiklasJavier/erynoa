//! # Event Engine
//!
//! Event-Verarbeitung und DAG-Management gemäß Κ9-Κ12.
//!
//! ## Axiom-Referenz
//!
//! - **Κ9 (Kausale Struktur)**: `ℂ = (E, ⊲)` ist ein DAG
//! - **Κ10 (Bezeugung-Finalität)**: `⟦e⟧ → □⟦e⟧`
//! - **Κ11 (Prozess-Korrektheit)**: `{pre} Π {post}`
//! - **Κ12 (Event-Erzeugung)**: `∀Π : ⟦Π⟧ → Δ|ℂ| ≥ 1`
//!
//! ## ExecutionContext-Integration (Phase 3)
//!
//! Die `*_with_ctx` Methoden integrieren mit dem ExecutionContext für:
//! - Gas-Accounting (Κ15b)
//! - Event-Emission über Context
//! - Invarianten-Prüfungen

use crate::core::engine::event_gas;
use crate::domain::unified::{Cost, FinalityLevel as UnifiedFinalityLevel, UniversalId};
use crate::domain::{Event, EventId, FinalityLevel, DID};
use crate::execution::{ExecutionContext, ExecutionError, ExecutionResult};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Fehler bei Event-Verarbeitung
#[derive(Debug, Error)]
pub enum EventError {
    #[error("Parent event not found: {0}")]
    ParentNotFound(EventId),

    #[error("Signature validation failed")]
    InvalidSignature,

    #[error("Event would create cycle in DAG")]
    CycleDetected,

    #[error("Event already exists: {0}")]
    DuplicateEvent(EventId),

    #[error("Invalid event payload")]
    InvalidPayload(String),
}

/// Ergebnis eines Event-Operations
pub type EventResult<T> = Result<T, EventError>;

/// Event Engine - verarbeitet und validiert Events im DAG (Κ9-Κ12)
///
/// ```text
/// ┌──────────────────────────────────────────────────────────────┐
/// │                     EventEngine                              │
/// │                                                              │
/// │  ┌──────────┐    ┌──────────┐    ┌──────────────┐          │
/// │  │ Validate │───▶│  Store   │───▶│ Notify       │          │
/// │  │ (Κ9,Κ10) │    │  (DAG)   │    │ (Witnesses)  │          │
/// │  └──────────┘    └──────────┘    └──────────────┘          │
/// └──────────────────────────────────────────────────────────────┘
/// ```
pub struct EventEngine {
    /// In-Memory Event-Index (für Production: über Repository)
    events: HashMap<EventId, Event>,

    /// Kinder-Index (child → parents)
    children_index: HashMap<EventId, HashSet<EventId>>,

    /// Genesis-Events (keine Parents)
    genesis_events: HashSet<EventId>,

    /// Konfiguration
    config: EventEngineConfig,
}

/// Konfiguration für EventEngine
#[derive(Debug, Clone)]
pub struct EventEngineConfig {
    /// Minimum Witnesses für Witnessed-Level
    pub min_witnesses: usize,
    /// Trust-Schwelle für Witnesses
    pub witness_trust_threshold: f64,
    /// Maximum Parents pro Event
    pub max_parents: usize,
}

impl Default for EventEngineConfig {
    fn default() -> Self {
        Self {
            min_witnesses: 3,
            witness_trust_threshold: 0.5,
            max_parents: 10,
        }
    }
}

impl EventEngine {
    /// Erstelle neue EventEngine
    pub fn new(config: EventEngineConfig) -> Self {
        Self {
            events: HashMap::new(),
            children_index: HashMap::new(),
            genesis_events: HashSet::new(),
            config,
        }
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(EventEngineConfig::default())
    }

    /// Κ9: Validiere Event-Struktur (DAG-Integrität)
    pub fn validate_structure(&self, event: &Event) -> EventResult<()> {
        // Prüfe ob Parents existieren
        for parent_id in &event.parents {
            if !self.events.contains_key(parent_id) {
                return Err(EventError::ParentNotFound(parent_id.clone()));
            }
        }

        // Prüfe auf Zyklen (Event darf nicht sein eigener Vorfahr sein)
        if self.would_create_cycle(&event.id, &event.parents) {
            return Err(EventError::CycleDetected);
        }

        // Prüfe max Parents
        if event.parents.len() > self.config.max_parents {
            return Err(EventError::InvalidPayload(format!(
                "Too many parents: {} > {}",
                event.parents.len(),
                self.config.max_parents
            )));
        }

        Ok(())
    }

    /// Prüft ob das Hinzufügen eines Events einen Zyklus erzeugen würde
    fn would_create_cycle(&self, event_id: &EventId, parents: &[EventId]) -> bool {
        // BFS von jedem Parent zurück zum Event
        for parent_id in parents {
            let mut visited = HashSet::new();
            let mut queue = vec![parent_id.clone()];

            while let Some(current) = queue.pop() {
                if &current == event_id {
                    return true;
                }

                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current.clone());

                // Hole Children (Events die dieses als Parent haben)
                if let Some(children) = self.children_index.get(&current) {
                    for child in children {
                        queue.push(child.clone());
                    }
                }
            }
        }

        false
    }

    /// Κ12: Füge Event zum DAG hinzu
    pub fn add_event(&mut self, event: Event) -> EventResult<EventId> {
        // Prüfe Duplikat
        if self.events.contains_key(&event.id) {
            return Err(EventError::DuplicateEvent(event.id.clone()));
        }

        // Validiere Struktur (Κ9)
        self.validate_structure(&event)?;

        let event_id = event.id.clone();

        // Update Children-Index
        for parent_id in &event.parents {
            self.children_index
                .entry(parent_id.clone())
                .or_default()
                .insert(event_id.clone());
        }

        // Genesis-Event?
        if event.parents.is_empty() {
            self.genesis_events.insert(event_id.clone());
        }

        // Speichere Event
        self.events.insert(event_id.clone(), event);

        Ok(event_id)
    }

    /// Hole Event by ID
    pub fn get_event(&self, id: &EventId) -> Option<&Event> {
        self.events.get(id)
    }

    /// Hole alle Events eines Authors
    pub fn get_events_by_author(&self, author: &UniversalId) -> Vec<&Event> {
        self.events
            .values()
            .filter(|e| &e.author == author)
            .collect()
    }

    /// Berechne kausale Geschichte |ℂ(s)| für ein Subject
    pub fn causal_history_size(&self, did: &DID) -> u64 {
        self.get_events_by_author(&did.id).len() as u64
    }

    /// Κ10: Update Finalitätslevel
    pub fn update_finality(&mut self, event_id: &EventId, level: FinalityLevel) -> EventResult<()> {
        let event = self
            .events
            .get_mut(event_id)
            .ok_or_else(|| EventError::ParentNotFound(event_id.clone()))?;

        // Finalität kann nur aufsteigen, nie absteigen (Permanenz)
        if level > event.finality.level {
            event.finality.level = level;
        }

        Ok(())
    }

    /// Hole alle Events mit bestimmtem Finalitätslevel
    pub fn get_events_by_finality(&self, min_level: FinalityLevel) -> Vec<&Event> {
        self.events
            .values()
            .filter(|e| e.finality.level >= min_level)
            .collect()
    }

    /// Berechne topologische Ordnung (für Processing)
    pub fn topological_order(&self) -> Vec<&Event> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        fn visit<'a>(
            event_id: &EventId,
            events: &'a HashMap<EventId, Event>,
            visited: &mut HashSet<EventId>,
            temp_mark: &mut HashSet<EventId>,
            result: &mut Vec<&'a Event>,
        ) {
            if visited.contains(event_id) {
                return;
            }
            if temp_mark.contains(event_id) {
                return; // Zyklus (sollte nicht passieren)
            }

            temp_mark.insert(event_id.clone());

            if let Some(event) = events.get(event_id) {
                for parent in &event.parents {
                    visit(parent, events, visited, temp_mark, result);
                }
                temp_mark.remove(event_id);
                visited.insert(event_id.clone());
                result.push(event);
            }
        }

        for event_id in self.events.keys() {
            visit(
                event_id,
                &self.events,
                &mut visited,
                &mut temp_mark,
                &mut result,
            );
        }

        result
    }

    /// Statistiken
    pub fn stats(&self) -> EventEngineStats {
        let finality_counts = self.events.values().fold([0usize; 5], |mut acc, e| {
            match e.finality.level {
                FinalityLevel::Nascent => acc[0] += 1,
                FinalityLevel::Validated => acc[1] += 1,
                FinalityLevel::Witnessed => acc[2] += 1,
                FinalityLevel::Anchored => acc[3] += 1,
                FinalityLevel::Eternal => acc[4] += 1,
            }
            acc
        });

        EventEngineStats {
            total_events: self.events.len(),
            genesis_events: self.genesis_events.len(),
            nascent: finality_counts[0],
            validated: finality_counts[1],
            witnessed: finality_counts[2],
            anchored: finality_counts[3],
            eternal: finality_counts[4],
        }
    }

    // =========================================================================
    // ExecutionContext-Integration (Phase 3.1)
    // =========================================================================

    /// Κ9: Validiere Event-Struktur mit Gas-Accounting
    ///
    /// Gas: VALIDATE + PARENT_LOOKUP × num_parents + CYCLE_CHECK
    pub fn validate_structure_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        event: &Event,
    ) -> ExecutionResult<()> {
        // Gas für Basis-Validierung
        ctx.consume_gas(event_gas::VALIDATE)?;

        // Gas für Parent-Lookups
        let parent_cost = event_gas::PARENT_LOOKUP * event.parents.len() as u64;
        ctx.consume_gas(parent_cost)?;

        // Gas für Zyklus-Check
        if !event.parents.is_empty() {
            ctx.consume_gas(event_gas::CYCLE_CHECK)?;
        }

        // Legacy-Validierung aufrufen
        self.validate_structure(event).map_err(|e| match e {
            EventError::ParentNotFound(id) => ExecutionError::NotFound {
                resource_type: "Event".into(),
                id: format!("{:?}", id),
            },
            EventError::CycleDetected => ExecutionError::CausalOrderViolation {
                event_id: format!("{:?}", event.id),
                parent_id: "cycle".into(),
            },
            EventError::InvalidPayload(msg) => ExecutionError::InvalidInput(msg),
            _ => ExecutionError::Internal(e.to_string()),
        })?;

        // Cost tracken
        let total_gas = event_gas::VALIDATE + parent_cost + event_gas::CYCLE_CHECK;
        ctx.track_cost(Cost::new(total_gas, 0, 0.0));

        Ok(())
    }

    /// Κ12: Füge Event zum DAG hinzu mit Gas-Accounting
    ///
    /// Gas: VALIDATE + ADD_TO_DAG + PARENT_LOOKUP × num_parents
    pub fn add_event_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        event: Event,
    ) -> ExecutionResult<EventId> {
        // Validiere zuerst
        self.validate_structure_with_ctx(ctx, &event)?;

        // Gas für DAG-Insertion
        ctx.consume_gas(event_gas::ADD_TO_DAG)?;

        // Legacy add_event aufrufen (ohne erneute Validierung)
        let event_id = event.id.clone();

        // Prüfe Duplikat
        if self.events.contains_key(&event_id) {
            return Err(ExecutionError::InvalidInput(format!(
                "Duplicate event: {:?}",
                event_id
            )));
        }

        // Update Children-Index
        for parent_id in &event.parents {
            self.children_index
                .entry(parent_id.clone())
                .or_default()
                .insert(event_id.clone());
        }

        // Genesis-Event?
        if event.parents.is_empty() {
            self.genesis_events.insert(event_id.clone());
        }

        // Speichere Event
        self.events.insert(event_id.clone(), event.clone());

        // Event über Context emittieren (Κ12)
        ctx.emit_raw("event.added", event_id.as_bytes());

        // Lamport-Clock updaten
        ctx.state.tick();

        // Cost tracken
        ctx.track_cost(Cost::new(event_gas::ADD_TO_DAG, 0, 0.0));

        Ok(event_id)
    }

    /// Κ10: Update Finalitätslevel mit ExecutionContext
    ///
    /// Prüft Permanenz (Finality kann nur aufsteigen) und trackt Kosten.
    pub fn update_finality_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        event_id: &EventId,
        level: FinalityLevel,
    ) -> ExecutionResult<()> {
        ctx.consume_gas(50)?;

        let event = self
            .events
            .get_mut(event_id)
            .ok_or_else(|| ExecutionError::NotFound {
                resource_type: "Event".into(),
                id: format!("{:?}", event_id),
            })?;

        // Κ10: Finality kann nur aufsteigen, nie absteigen (Permanenz)
        if level < event.finality.level {
            return Err(ExecutionError::FinalityRegression {
                event_id: format!("{:?}", event_id),
                old_level: event.finality.level as u8,
                new_level: level as u8,
            });
        }

        if level > event.finality.level {
            event.finality.level = level;
            ctx.emit_raw("event.finality_updated", event_id.as_bytes());
        }

        Ok(())
    }

    /// Verarbeite mehrere Events als Batch mit Gas-Budget
    ///
    /// Stoppt wenn Gas erschöpft ist und gibt Anzahl verarbeiteter Events zurück.
    pub fn process_batch_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        events: Vec<Event>,
    ) -> ExecutionResult<usize> {
        let mut processed = 0;

        for event in events {
            // Prüfe ob genug Gas für mindestens ein Event
            let estimated_gas = event_gas::VALIDATE
                + event_gas::ADD_TO_DAG
                + event_gas::PARENT_LOOKUP * event.parents.len() as u64;

            if ctx.gas_remaining < estimated_gas {
                break;
            }

            match self.add_event_with_ctx(ctx, event) {
                Ok(_) => processed += 1,
                Err(ExecutionError::GasExhausted { .. }) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(processed)
    }
}

/// Statistiken der EventEngine
#[derive(Debug, Clone)]
pub struct EventEngineStats {
    pub total_events: usize,
    pub genesis_events: usize,
    pub nascent: usize,
    pub validated: usize,
    pub witnessed: usize,
    pub anchored: usize,
    pub eternal: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EventPayload;

    #[test]
    fn test_add_genesis_event() {
        let mut engine = EventEngine::default();
        let did = DID::new_self(b"alice");
        let event = Event::genesis(did.clone(), "pubkey".to_string());

        let id = engine.add_event(event).unwrap();
        assert!(engine.genesis_events.contains(&id));
        assert_eq!(engine.stats().genesis_events, 1);
    }

    #[test]
    fn test_add_event_with_parent() {
        let mut engine = EventEngine::default();
        let did = DID::new_self(b"alice");

        // Genesis
        let genesis = Event::genesis(did.clone(), "pubkey".to_string());
        let genesis_id = engine.add_event(genesis).unwrap();

        // Child-Event
        let child = Event::new(
            did.clone(),
            EventPayload::Transfer {
                from: did.clone(),
                to: DID::new_self(b"bob"),
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            vec![genesis_id.clone()],
        );

        let child_id = engine.add_event(child).unwrap();

        // Parent sollte Child in children_index haben
        assert!(engine
            .children_index
            .get(&genesis_id)
            .unwrap()
            .contains(&child_id));
    }

    #[test]
    fn test_reject_missing_parent() {
        let mut engine = EventEngine::default();
        let did = DID::new_self(b"alice");

        let event = Event::new(
            did.clone(),
            EventPayload::Transfer {
                from: did.clone(),
                to: DID::new_self(b"bob"),
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            vec![EventId::new("nonexistent")],
        );

        let result = engine.add_event(event);
        assert!(matches!(result, Err(EventError::ParentNotFound(_))));
    }

    #[test]
    fn test_causal_history_size() {
        let mut engine = EventEngine::default();
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        // Alice: 3 Events
        let e1 = Event::genesis(alice.clone(), "pk".to_string());
        let id1 = engine.add_event(e1).unwrap();

        let e2 = Event::new(
            alice.clone(),
            EventPayload::Custom {
                event_type: "test".to_string(),
                data: serde_json::Value::Null,
            },
            vec![id1.clone()],
        );
        let id2 = engine.add_event(e2).unwrap();

        let e3 = Event::new(
            alice.clone(),
            EventPayload::Custom {
                event_type: "test".to_string(),
                data: serde_json::Value::Null,
            },
            vec![id2],
        );
        engine.add_event(e3).unwrap();

        // Bob: 1 Event
        let e4 = Event::genesis(bob.clone(), "pk".to_string());
        engine.add_event(e4).unwrap();

        assert_eq!(engine.causal_history_size(&alice), 3);
        assert_eq!(engine.causal_history_size(&bob), 1);
    }

    // =========================================================================
    // ExecutionContext Tests (Phase 3.1)
    // =========================================================================

    #[test]
    fn test_add_event_with_ctx() {
        let mut engine = EventEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();
        let did = DID::new_self(b"alice");

        let event = Event::genesis(did.clone(), "pubkey".to_string());
        let initial_gas = ctx.gas_remaining;

        let id = engine.add_event_with_ctx(&mut ctx, event).unwrap();

        // Event wurde hinzugefügt
        assert!(engine.get_event(&id).is_some());

        // Gas wurde verbraucht
        assert!(ctx.gas_remaining < initial_gas);

        // Event wurde emittiert
        assert!(!ctx.emitted_events.is_empty());
    }

    #[test]
    fn test_validate_with_ctx_parent_not_found() {
        let engine = EventEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();
        let did = DID::new_self(b"alice");

        let event = Event::new(
            did.clone(),
            EventPayload::Custom {
                event_type: "test".to_string(),
                data: serde_json::Value::Null,
            },
            vec![EventId::new("nonexistent")],
        );

        let result = engine.validate_structure_with_ctx(&mut ctx, &event);
        assert!(matches!(result, Err(ExecutionError::NotFound { .. })));
    }

    #[test]
    fn test_update_finality_with_ctx() {
        let mut engine = EventEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();
        let did = DID::new_self(b"alice");

        // Genesis hinzufügen
        let event = Event::genesis(did.clone(), "pubkey".to_string());
        let id = engine.add_event_with_ctx(&mut ctx, event).unwrap();

        // Finality aufsteigen: Nascent -> Validated -> Witnessed
        engine
            .update_finality_with_ctx(&mut ctx, &id, FinalityLevel::Validated)
            .unwrap();
        engine
            .update_finality_with_ctx(&mut ctx, &id, FinalityLevel::Witnessed)
            .unwrap();

        assert_eq!(
            engine.get_event(&id).unwrap().finality,
            FinalityLevel::Witnessed
        );
    }

    #[test]
    fn test_finality_regression_prevented() {
        let mut engine = EventEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();
        let did = DID::new_self(b"alice");

        let event = Event::genesis(did.clone(), "pubkey".to_string());
        let id = engine.add_event_with_ctx(&mut ctx, event).unwrap();

        // Auf Witnessed setzen
        engine
            .update_finality_with_ctx(&mut ctx, &id, FinalityLevel::Witnessed)
            .unwrap();

        // Regression versuchen: Witnessed -> Nascent (verboten nach Κ10)
        let result = engine.update_finality_with_ctx(&mut ctx, &id, FinalityLevel::Nascent);
        assert!(matches!(
            result,
            Err(ExecutionError::FinalityRegression { .. })
        ));
    }

    #[test]
    fn test_gas_exhaustion_stops_batch() {
        let mut engine = EventEngine::default();
        // Nur genug Gas für ca. 2 Events - use minimal() and modify gas
        let mut ctx = ExecutionContext::minimal();
        ctx.gas_remaining = 500; // Niedrig für den Test
        ctx.gas_initial = 500;

        let events: Vec<Event> = (0..10)
            .map(|i| Event::genesis(DID::new_self(&format!("user{}", i)), "pubkey".to_string()))
            .collect();

        let processed = engine.process_batch_with_ctx(&mut ctx, events).unwrap();

        // Nicht alle Events sollten verarbeitet werden
        assert!(processed < 10);
        assert!(processed > 0);
    }
}
