//! # ECLVM - Erynoa Configuration Language Virtual Machine
//!
//! Eine Stack-basierte, Gas-metered VM für deterministische Policy-Ausführung.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         ECLVM Pipeline                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │   ┌─────────┐    ┌──────────┐    ┌──────────┐    ┌─────────┐   │
//! │   │   ECL   │───▶│  Parser  │───▶│ Compiler │───▶│Bytecode │   │
//! │   │  Text   │    │ (Lexer)  │    │(AST→Op)  │    │(OpCode) │   │
//! │   └─────────┘    └──────────┘    └──────────┘    └────┬────┘   │
//! │                                                       │        │
//! │   ┌───────────────────────────────────────────────────┘        │
//! │   │                                                            │
//! │   ▼                                                            │
//! │   ┌─────────────────────────────────────────────────────────┐  │
//! │   │                    ECLVM Runtime                        │  │
//! │   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌───────────┐   │  │
//! │   │  │  Stack  │  │   IP    │  │   Gas   │  │   Host    │   │  │
//! │   │  │[Value]  │  │(usize)  │  │ Meter   │  │ Interface │   │  │
//! │   │  └─────────┘  └─────────┘  └─────────┘  └───────────┘   │  │
//! │   │                                              │          │  │
//! │   │                                              ▼          │  │
//! │   │                                    ┌─────────────────┐  │  │
//! │   │                                    │  Erynoa Core    │  │  │
//! │   │                                    │ (Trust, Events) │  │  │
//! │   │                                    └─────────────────┘  │  │
//! │   └─────────────────────────────────────────────────────────┘  │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Verwendung
//!
//! ```rust,ignore
//! use erynoa_api::eclvm::{ECLVM, OpCode, Value, GasMeter};
//!
//! // Manuelles Bytecode-Programm: 5 + 3 = 8
//! let program = vec![
//!     OpCode::PushConst(Value::Number(5.0)),
//!     OpCode::PushConst(Value::Number(3.0)),
//!     OpCode::Add,
//!     OpCode::Return,
//! ];
//!
//! let mut vm = ECLVM::new(program, 1000);
//! let result = vm.run().unwrap();
//! assert_eq!(result, Value::Number(8.0));
//! ```

pub mod ast;
pub mod bridge;
pub mod bytecode;
#[cfg(feature = "cli")]
pub mod cli;
pub mod compiler;
pub mod erynoa_host;
pub mod mana;
pub mod optimizer;
pub mod parser;
pub mod programmable_gateway;
pub mod runtime;
pub mod stdlib;

// Re-exports für einfachen Zugriff
pub use bridge::{CoreToEclvm, EclvmToCore, InterpretError};
pub use bytecode::{OpCode, Value};
#[cfg(feature = "cli")]
pub use cli::{run_cli, Cli, Commands};
pub use erynoa_host::{ErynoaHost, PolicyContext};
pub use mana::{BandwidthTier, ManaAccount, ManaConfig, ManaManager, ManaStatus};
pub use optimizer::{OptimizationStats, Optimizer};
pub use programmable_gateway::{
    CompiledPolicy, GatewayDecision, ProgrammableGateway, StandardPolicies,
};
pub use runtime::{gas::GasMeter, host::HostInterface, vm::ECLVM};
pub use stdlib::{PolicyBuilder, StdLib};
