//! ECL CLI Binary
//!
//! Dieses Binary bietet eine Kommandozeilen-Schnittstelle für ECL.
//!
//! ## Installation
//!
//! ```bash
//! cargo install --path . --features cli --bin ecl
//! ```
//!
//! ## Usage
//!
//! ```bash
//! ecl repl              # Interaktive REPL
//! ecl eval "2 + 3"      # Expression evaluieren
//! ecl run policy.ecl    # Policy ausführen
//! ecl compile policy.ecl -o policy.eclc  # Kompilieren
//! ecl check policy.ecl  # Syntax prüfen
//! ```

use anyhow::Result;

fn main() -> Result<()> {
    erynoa_api::eclvm::run_cli()
}
