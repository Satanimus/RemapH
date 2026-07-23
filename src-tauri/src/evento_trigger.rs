// ======================================================
// 🎯 EventoTrigger RemapH V3
// ------------------------------------------------------
// Representa un trigger ya analizado.
//
// Flujo:
//
// InputEvent físico
//        ↓
// BufferEventos
//        ↓
// AnalizadorTrigger
//        ↓
// EventoTrigger
//
// Aquí ya no existen:
//   - Down.
//   - Up.
//   - Tiempos.
//   - Buffer.
//
// Solo existe la intención detectada:
//   - Simple.
//   - Doble.
//   - Mantenido.
//
// También conserva:
//   - Modificadores activos.
// ======================================================

use crate::eventos::InputId;
use crate::perfilcache::CondicionTrigger;
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
