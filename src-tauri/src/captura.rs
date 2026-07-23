// ======================================================
// 🎹 Captura RemapH V3
// ------------------------------------------------------
// Modo temporal de captura.
//
// No lee dispositivos.
// No conoce Windows.
// No conoce Interception.
//
// Recibe EventoTrigger ya analizado.
//
// Flujo:
//
// InputEvent
//      ↓
// AnalizadorTrigger
//      ↓
// Captura
//      ↓
// UI
// ======================================================

use std::sync::{Mutex, OnceLock};

use crate::evento_trigger::EventoTrigger;

// ======================================================
// 🧠 ESTADO
// ======================================================

struct EstadoCaptura {
    activa: bool,

    ultimo: Option<EventoTrigger>,
}

static CAPTURA: OnceLock<Mutex<EstadoCaptura>> = OnceLock::new();

// ======================================================
// 🔒 ESTADO
// ======================================================

fn estado() -> &'static Mutex<EstadoCaptura> {
    CAPTURA.get_or_init(|| {
        Mutex::new(EstadoCaptura {
            activa: false,

            ultimo: None,
        })
    })
}

// ======================================================
// 🚀 INICIAR
// ======================================================

pub fn iniciar() {
    let mut captura = estado().lock().unwrap();

    captura.activa = true;

    captura.ultimo = None;
}

// ======================================================
// 🛑 FINALIZAR
// ======================================================

pub fn finalizar() {
    estado().lock().unwrap().activa = false;
}

// ======================================================
// ❓ ACTIVA
// ======================================================

pub fn activa() -> bool {
    estado().lock().unwrap().activa
}

// ======================================================
// 📥 RECIBIR TRIGGER
// ======================================================

pub fn recibir(evento: EventoTrigger) {
    let mut captura = estado().lock().unwrap();

    if !captura.activa {
        return;
    }

    captura.ultimo = Some(evento);

    captura.activa = false;
}

// ======================================================
// 📤 OBTENER RESULTADO
// ======================================================

pub fn obtener() -> Option<EventoTrigger> {
    estado().lock().unwrap().ultimo.clone()
}
