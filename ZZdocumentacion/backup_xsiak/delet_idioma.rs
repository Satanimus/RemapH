// ======================================================
// 🌐 IDIOMA RemapH V3
// ======================================================
// ETAPA 1 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Define el lenguaje interno que se guarda en disco: el
// Input de perfil (fuente + control como texto plano).
// Es la forma en que el JSON representa una entrada,
// distinta de InputId (que usa el motor en tiempo real).
// El constructor permite instanciarlo fácilmente.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// perfil_json (lo usa para representar Trigger.entrada)
// perfil_ui (convierte EntradaUI ↔ Input)
// compilador (lo convierte a InputId para la Cache)
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// fuente: texto plano (ej: "keyboard", "mouse")
// control: texto plano (ej: "A", "LeftButton")
// Ejemplo:
// Input::nuevo("keyboard", "A")
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// Input
// Ejemplo:
// Input {
//     fuente: "keyboard",
//     control: "A",
// }
// ------------------------------------------------------
// 5. Funciones del archivo
// Input::nuevo()
//     Construye un Input a partir de fuente y control.
// ------------------------------------------------------
// Transformación que realiza
// perfil.json (texto plano)
//     ↓
// Input { fuente, control }
//     ↓
// compilador::convertir_input() → InputId
// ======================================================

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Input {
    pub fuente: String,

    pub control: String,
}

impl Input {
    pub fn nuevo(fuente: &str, control: &str) -> Self {
        Self {
            fuente: fuente.to_string(),

            control: control.to_string(),
        }
    }
}
