// ======================================================
// 🎹 Capturador Trigger RemapH V3
// ------------------------------------------------------
// Construye un EventoTrigger para edición.
//
// No consulta Cache.
// No conoce Runtime.
// No consume eventos.
// No libera eventos.
//
// Flujo:
//
// Input físico
//      ↓
// CapturadorTrigger
//      ↓
// EventoTrigger
//      ↓
// Captura
// ======================================================

use crate::evento_trigger::EventoTrigger;
use crate::eventos::{InputEvent, InputId, InputState};

// ======================================================
// 🧠 CAPTURADOR
// ======================================================

pub struct CapturadorTrigger {
    eventos: Vec<InputEvent>,
}

// ======================================================
// 🚀 CREAR
// ======================================================

impl CapturadorTrigger {
    pub fn nuevo() -> Self {
        Self {
            eventos: Vec::new(),
        }
    }

    // ==================================================
    // 📥 RECIBIR EVENTO
    // ==================================================

    pub fn recibir(&mut self, evento: InputEvent) {
        self.eventos.push(evento);
    }

    // ==================================================
    // 🎯 CONSTRUIR TRIGGER
    // ==================================================

    pub fn construir(&self) -> Option<EventoTrigger> {
        let activos = self.inputs_down();

        if activos.is_empty() {
            return None;
        }

        let gatillo = activos.last()?.clone();

        let modificadores = if activos.len() > 1 {
            activos[..activos.len() - 1].to_vec()
        } else {
            Vec::new()
        };

        // De momento siempre Simple.
        // Más adelante aquí se calcularán
        // Doble y Mantenido.

        Some(EventoTrigger::simple(modificadores, gatillo))
    }

    // ==================================================
    // ⬇️ INPUTS DOWN
    // ==================================================

    fn inputs_down(&self) -> Vec<InputId> {
        self.eventos
            .iter()
            .filter(|evento| evento.state == InputState::Down)
            .map(|evento| evento.input.clone())
            .collect()
    }

    // ==================================================
    // 🧹 LIMPIAR
    // ==================================================

    pub fn limpiar(&mut self) {
        self.eventos.clear();
    }
}
