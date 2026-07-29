// ======================================================
// 🎯 evento_trigger RemapH V3
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Representa un Trigger completamente interpretado.
// Aquí ya desaparecieron:
// • Down. • Up. • Instantes. • Buffer. • Eventos físicos.
//
// Sólo existe la intención detectada.
// Runtime nunca analiza tiempos. Sólo recibe este objeto.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// AnalizadorTrigger.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// El AnalizadorTrigger ya determinó:
// • Entrada completa.  • Condición.
//
// Ejemplo:
// Entrada:  Ctrl, Shift, A
// Condición:  Doble
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// EventoTrigger.
// Ejemplo:
// EventoTrigger {
//     entrada: [
//         keyboard:LeftCtrl,
//         keyboard:LeftShift,
//         keyboard:A,  ],
//  condicion: Doble, }
// ------------------------------------------------------
// 5. Funciones del archivo
// nuevo()
//     Construye un EventoTrigger.
// ------------------------------------------------------
// Transformación que realiza
// InputEvents físicos
//          ↓
// AnalizadorTrigger
//          ↓
// EventoTrigger {
//     entrada,
//     condicion, }
// ======================================================

use crate::eventos::InputId;
use crate::perfil_cache::CondicionTrigger;

use serde::{Deserialize, Serialize};

// ======================================================
// 🎯 EVENTO TRIGGER
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventoTrigger {
    pub entrada: Vec<InputId>,

    pub condicion: CondicionTrigger,
}

// ======================================================
// 🏗️ CONSTRUCTOR
// ======================================================

impl EventoTrigger {
    pub fn nuevo(entrada: Vec<InputId>, condicion: CondicionTrigger) -> Self {
        Self { entrada, condicion }
    }
}
