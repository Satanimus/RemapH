// ======================================================
// 🌐 Idioma RemapH V3
// ------------------------------------------------------
// Representa el lenguaje interno del programa.
//
// El JSON guarda este idioma.
// UI y Backend traducen hacia/desde él.
// Runtime solo recibe InputId.
// ======================================================

use serde::{Deserialize, Serialize};

// ======================================================
// 🆔 INPUT INTERNO
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Input {
    pub fuente: String,

    pub control: String,
}

// ======================================================
// 🏗️ CREAR INPUT
// ======================================================

impl Input {
    pub fn nuevo(fuente: &str, control: &str) -> Self {
        Self {
            fuente: fuente.to_string(),

            control: control.to_string(),
        }
    }
}
