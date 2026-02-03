//! Erynoa egui Debugger – Binary Einstiegspunkt.
//!
//! Start: `cargo run --bin erynoa-debug --features debug`
//!
//! Öffnet ein natives egui-Fenster mit Side-Panel (Uptime, Health, Warnings),
//! Tabs (Overview, StateGraph, Trust, Events, P2P, Realms, Execution, Logs)
//! und Bottom-Bar (FPS, Debug Mode). Nutzt einen lokalen UnifiedState + StateCoordinator
//! (ohne Storage); für Live-Daten an laufendem Backend später z. B. HTTP-Polling ergänzbar.

use eframe::egui;
use erynoa_api::core::{create_unified_state, StateCoordinator};
use erynoa_api::debug::{DebugState, ErynoaDebugApp};
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let state = create_unified_state();
    let coordinator = Arc::new(StateCoordinator::new(state.clone()));
    let debug_state = DebugState::new(state, coordinator);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Erynoa Debugger",
        options,
        Box::new(|_cc| Ok(Box::new(ErynoaDebugApp::new(debug_state)))),
    )
}
