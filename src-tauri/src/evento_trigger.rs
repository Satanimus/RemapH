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
// ======================================================

use crate::eventos::InputId;
use crate::perfilcache::CondicionTrigger;

// ======================================================
// 🎯 EVENTO TRIGGER
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventoTrigger {
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

    pub fn simple(gatillo: InputId) -> Self {
        Self {
            gatillo,

            condicion: CondicionTrigger::Simple,
        }
    }

    // ==================================================
    // DOBLE
    // ==================================================

    pub fn doble(gatillo: InputId) -> Self {
        Self {
            gatillo,

            condicion: CondicionTrigger::Doble,
        }
    }

    // ==================================================
    // MANTENIDO
    // ==================================================

    pub fn mantenido(gatillo: InputId) -> Self {
        Self {
            gatillo,

            condicion: CondicionTrigger::Mantenido,
        }
    }
}
