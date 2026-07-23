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
// Responsabilidades futuras:
//
//   - Detectar Simple.
//   - Detectar Doble.
//   - Detectar Mantenido.
//   - Mantener timeline.
//   - Resolver modificadores activos.
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
                // Guarda modificadores activos.
                //
                // La identificación definitiva de modificador
                // dependerá del cache/perfil.
                self.modificadores_activos.push(evento.input.clone());

                Some(EventoTrigger::simple(
                    self.modificadores_activos.clone(),
                    evento.input,
                ))
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
