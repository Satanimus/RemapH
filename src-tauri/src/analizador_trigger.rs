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
// Responsabilidades:
//
//   - Detectar gatillo.
//   - Mantener modificadores activos.
//   - Preparar Simple/Doble/Mantenido.
//
// No ejecuta acciones.
// No conoce Runtime.
// ======================================================

use crate::cache;
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
                let input = evento.input.clone();

                // --------------------------------------
                // Es modificador
                // --------------------------------------

                if cache::es_modificador(&input) {
                    if !self.modificadores_activos.contains(&input) {
                        self.modificadores_activos.push(input);
                    }

                    return None;
                }

                // --------------------------------------
                // Es gatillo
                // --------------------------------------

                println!("[ANALIZADOR] Gatillo -> {:?}", input);

                Some(EventoTrigger::simple(
                    self.modificadores_activos.clone(),
                    input,
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
