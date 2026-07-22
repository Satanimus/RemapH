// ======================================================
// 🧠 Buffer eventos RemapH V3
// ------------------------------------------------------
// Convierte eventos físicos en eventos lógicos.
//
// Recibe:
//     Down
//     Up
//     Pulse
//
// Analiza:
//
//     • Simple
//     • Doble
//     • Mantenido
//     • Rueda
//     • Modificadores
//     • Gatillo
//
// Devuelve:
//
//     None
//         -> todavía esperando.
//
//     Some(InputEvent)
//         -> evento completamente analizado.
//
// Este módulo será compartido por:
//
//     • Backend Portable
//     • Backend Interception
// ======================================================

use crate::eventos::InputEvent;

pub struct BufferEventos {

}

impl BufferEventos {

    // ==================================================
    // 🚀 CREAR
    // ==================================================

    pub fn nuevo() -> Self {

        Self {

        }

    }

    // ==================================================
    // 📥 RECIBIR EVENTO
    // ==================================================

    pub fn recibir(

        &mut self,

        evento: InputEvent,

    ) -> Option<InputEvent> {

        Some(evento)

    }

}