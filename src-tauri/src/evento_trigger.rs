// ======================================================
// 🎯 EVENTOTRIGGER RemapH V3
// ======================================================
// ETAPA 2 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Representa un Trigger completamente interpretado.
// Aquí ya desaparecieron:
// - Down.
// - Up.
// - Instantes.
// - Buffer.
// - Eventos físicos.
//
// Sólo existe la intención detectada.
//
// El Runtime nunca analiza tiempos.
// Sólo recibe este objeto.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// AnalizadorTrigger.
//
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// El AnalizadorTrigger ya determinó:
//
// - Modificadores activos.
// - Gatillo.
// - Tipo de trigger.
//
// Ejemplo:
//
// Ctrl
// Shift
// A
// Doble
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// EventoTrigger.
//
// Ejemplo:
//
// EventoTrigger {
//     modificadores: [keyboard:LeftCtrl, keyboard:LeftShift],
//     gatillo: keyboard:A,
//     condicion: Doble,
// }
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// simple()
//     Construye un trigger Simple.
//
// doble()
//     Construye un trigger Doble.
//
// mantenido()
//     Construye un trigger Mantenido.
//
// ------------------------------------------------------
// Transformación que realiza
//
// InputEvents físicos
//          ↓
// AnalizadorTrigger
//          ↓
// EventoTrigger {
//     modificadores,
//     gatillo,
//     condicion,
// }
// ======================================================

use crate::eventos::InputId;
use crate::perfil_cache::CondicionTrigger;
use serde::{Deserialize, Serialize};

// ======================================================
// 🎯 EVENTO TRIGGER
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventoTrigger {
    pub modificadores: Vec<InputId>,

    pub gatillo: InputId,

    pub condicion: CondicionTrigger,
}

// ======================================================
// 🏗️ CONSTRUCTORES
// ======================================================

impl EventoTrigger {
    // ==================================================
    // SIMPLE
    // ==================================================

    pub fn simple(modificadores: Vec<InputId>, gatillo: InputId) -> Self {
        Self {
            modificadores,

            gatillo,

            condicion: CondicionTrigger::Simple,
        }
    }

    // ==================================================
    // DOBLE
    // ==================================================

    pub fn doble(modificadores: Vec<InputId>, gatillo: InputId) -> Self {
        Self {
            modificadores,

            gatillo,

            condicion: CondicionTrigger::Doble,
        }
    }

    // ==================================================
    // MANTENIDO
    // ==================================================

    pub fn mantenido(modificadores: Vec<InputId>, gatillo: InputId) -> Self {
        Self {
            modificadores,

            gatillo,

            condicion: CondicionTrigger::Mantenido,
        }
    }
}
