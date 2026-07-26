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

use std::time::Instant;

// ======================================================
// 🧠 CAPTURADOR
// ======================================================

pub struct CapturadorTrigger {
    eventos: Vec<InputEvent>,

    ultimo_evento: Option<Instant>,
}

// ======================================================
// 🚀 CREAR
// ======================================================

impl CapturadorTrigger {
    pub fn nuevo() -> Self {
        Self {
            eventos: Vec::new(),

            ultimo_evento: None,
        }
    }

    // ==================================================
    // 📥 RECIBIR EVENTO
    // ==================================================

    pub fn recibir(&mut self, evento: InputEvent) {
        self.eventos.push(evento);

        self.ultimo_evento = Some(Instant::now());
    }

    // ==================================================
    // ⏱️ FINALIZAR CAPTURA
    // ==================================================

    pub fn terminado(&self) -> bool {
        let Some(ultimo) = self.ultimo_evento else {
            return false;
        };

        ultimo.elapsed().as_millis() >= crate::config::tiempo_doble() as u128
    }

    // ==================================================
    // 🧪 PRUEBA CAPTURA
    // ==================================================

    pub fn eventos_completos(&self) -> bool {
        !self.eventos.is_empty()
    }

    // ==================================================
    // 🧪 Time out
    // ==================================================

    pub fn comprobar_timeout(&mut self) -> Option<EventoTrigger> {
        let Some(ultimo) = self.ultimo_evento else {
            return None;
        };

        if ultimo.elapsed().as_millis() < crate::config::tiempo_doble() as u128 {
            return None;
        }

        let trigger = self.construir();

        self.limpiar();

        trigger
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

        self.ultimo_evento = None;
    }
}
