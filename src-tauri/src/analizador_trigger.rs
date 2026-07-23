// ======================================================
// 🧠 Analizador Trigger RemapH V3
// ------------------------------------------------------
// Convierte eventos físicos en triggers lógicos.
//
// Entrada:
//
// InputEvent
//      ↓
// AnalizadorTrigger
//      ↓
// EventoTrigger
//
// Actualmente:
//
//   - Simple.
//   - Modificadores activos.
//
// Futuro:
//
//   - Doble.
//   - Mantenido.
//   - Timeline.
//
// No ejecuta acciones.
// No conoce Runtime.
// ======================================================

use crate::evento_trigger::EventoTrigger;
use crate::eventos::{InputEvent, InputId, InputState};

// ======================================================
// 🧠 ANALIZADOR
// ======================================================

pub struct AnalizadorTrigger {
    modificadores_activos: Vec<InputId>,
}

// ======================================================
// 🚀 CREAR
// ======================================================

impl AnalizadorTrigger {
    pub fn nuevo() -> Self {
        Self {
            modificadores_activos: Vec::new(),
        }
    }

    // ==================================================
    // 📥 PROCESAR
    // ==================================================

    pub fn procesar(&mut self, evento: InputEvent) -> Option<EventoTrigger> {
        match evento.state {
            // ------------------------------------------
            // ⬇️ DOWN
            // ------------------------------------------
            InputState::Down => {
                let gatillo = evento.input.clone();

                let modificadores = self.modificadores_activos.clone();

                self.modificadores_activos.push(gatillo.clone());

                Some(EventoTrigger::simple(modificadores, gatillo))
            }

            // ------------------------------------------
            // ⬆️ UP
            // ------------------------------------------
            InputState::Up => {
                self.modificadores_activos
                    .retain(|activo| activo != &evento.input);

                None
            }

            // ------------------------------------------
            // ⚡ PULSE
            // ------------------------------------------
            InputState::Pulse => Some(EventoTrigger::simple(
                self.modificadores_activos.clone(),
                evento.input,
            )),
        }
    }
}
