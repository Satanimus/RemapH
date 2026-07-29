// ======================================================
// 🎹 CapturadorTrigger RemapH V3
// ======================================================
// ETAPA 2 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Almacena temporalmente eventos físicos durante una
// captura de trigger.
//
// Su única responsabilidad es reunir los InputEvent
// recibidos y mantener el orden en que ocurrieron.
//
// CapturadorTrigger NO interpreta:
//
// - Combinaciones.
// - Doble pulsación.
// - Pulsación mantenida.
// - Condiciones.
// - Triggers.
//
// Tampoco conoce:
//
// - Perfil.
// - Cache.
// - Runtime.
// - Acciones.
// - Macros.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe:
//
// InputEvent:
//
// - InputId.
// - Estado físico.
// - Instante.
//
// Ejemplo:
//
// keyboard:A
// Down
// 105263 ms
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Backend de captura:
//
// back_windows
// back_interception
// rdev
//
// Flujo:
//
// Entrada física
//      ↓
// Backend captura
//      ↓
// InputEvent
//      ↓
// CapturadorTrigger
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Entrega:
//
// Vec<InputEvent>
//
// Manteniendo:
//
// - Orden.
// - Estado.
// - Instante.
//
// Ejemplo:
//
// [
//   Ctrl Down,
//   A Down,
//   A Up,
//   Ctrl Up
// ]
//
// ------------------------------------------------------
// 5. ¿Quién recibe la información después?
//
// AnalizadorTrigger.
//
// Flujo:
//
// CapturadorTrigger
//      ↓
// Vec<InputEvent>
//      ↓
// AnalizadorTrigger
//      ↓
// Trigger interno
//      ↓
// Compilador
//      ↓
// Cache
//
// ------------------------------------------------------
// 6. Funciones del archivo
//
// nuevo()
//     Crea un capturador vacío.
//
// recibir()
//     Agrega un InputEvent recibido.
//
// eventos()
//     Devuelve los eventos acumulados.
//
// esta_vacio()
//     Indica si existe una captura.
//
// limpiar()
//     Elimina eventos almacenados.
//
// ------------------------------------------------------
// Transformación:
//
// Input físico
//      ↓
// InputEvent
//      ↓
// CapturadorTrigger
//      ↓
// Lista ordenada de eventos
//      ↓
// AnalizadorTrigger
//
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
