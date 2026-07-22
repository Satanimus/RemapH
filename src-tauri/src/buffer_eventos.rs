// ======================================================
// 🧠 Buffer eventos RemapH V3
// ------------------------------------------------------
//Responsabilidad única:
//
//Agregar eventos físicos.
//Entregar eventos físicos.
//Limpiarse cuando el AnalizadorTrigger lo indique.
//
//Nunca:
//
//Calcula tiempos.
//Detecta doble.
//Detecta mantenido.
//Decide cuándo borrar.
// ======================================================

use crate::eventos::InputEvent;

pub struct BufferEventos {}

impl BufferEventos {
    // ==================================================
    // 🚀 CREAR
    // ==================================================

    pub fn nuevo() -> Self {
        Self {}
    }

    // ==================================================
    // 📥 RECIBIR EVENTO
    // ==================================================

    pub fn recibir(&mut self, evento: InputEvent) -> Option<InputEvent> {
        Some(evento)
    }
}
