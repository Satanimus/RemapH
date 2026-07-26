// ======================================================
// 🎹 Capturador Trigger RemapH V3
// ------------------------------------------------------
// Graba InputEvent físicos durante una captura.
//
// No interpreta.
// No consulta Cache.
// No conoce Runtime.
// No construye EventoTrigger.
//
// Flujo:
//
// Input físico
//      ↓
// CapturadorTrigger
//      ↓
// Vec<InputEvent>
//      ↓
// AnalizadorTrigger
// ======================================================

use crate::eventos::InputEvent;

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
    // ⏱️ ESPERAR SILENCIO
    // --------------------------------------------------
    // Devuelve true cuando no han llegado nuevos eventos
    // durante el tiempo de espera configurado.
    // ==================================================

    pub fn comprobar_timeout(&self) -> bool {
        let Some(ultimo) = self.ultimo_evento else {
            return false;
        };

        ultimo.elapsed().as_millis() >= crate::config::tiempo_doble() as u128
    }

    // ==================================================
    // 📦 EVENTOS CAPTURADOS
    // ==================================================

    pub fn eventos(&self) -> &[InputEvent] {
        &self.eventos
    }

    // ==================================================
    // ❓ VACÍO
    // ==================================================

    pub fn esta_vacio(&self) -> bool {
        self.eventos.is_empty()
    }

    // ==================================================
    // 🧹 LIMPIAR
    // ==================================================

    pub fn limpiar(&mut self) {
        self.eventos.clear();

        self.ultimo_evento = None;
    }
}
