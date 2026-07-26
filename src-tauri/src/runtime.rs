// ======================================================
// 🧠 Runtime RemapH V3
// ------------------------------------------------------
// Ejecuta remapeos compilados.
// ======================================================

use std::sync::mpsc::Sender;

use crate::cache;
use crate::evento_trigger::EventoTrigger;
use crate::perfil_cache::AccionCache;

// ======================================================
// ⚙️ RESULTADO
// ======================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resultado {
    Pasar,

    Consumir,
}

// ======================================================
// 🧠 ESTADO
// ======================================================

pub struct Estado;

// ======================================================
// 🚀 CREAR
// ======================================================

impl Estado {
    pub fn nuevo() -> Self {
        Self
    }

    // ==================================================
    // 🎯 PROCESAR
    // ==================================================

    pub fn procesar(&mut self, evento: EventoTrigger, salida: &Sender<AccionCache>) -> Resultado {
        if !crate::estado::esta_activo() {
            return Resultado::Pasar;
        }

        let mut activos = evento.modificadores.clone();

        activos.push(evento.gatillo.clone());

        let Some(remapeo) = cache::buscar(&activos, &evento.gatillo) else {
            return Resultado::Pasar;
        };

        // =============================================
        // Verificación de condición
        // =============================================

        if remapeo.trigger.condicion != evento.condicion {
            return Resultado::Pasar;
        }

        salida.send(remapeo.accion).unwrap();

        Resultado::Consumir
    }
}
