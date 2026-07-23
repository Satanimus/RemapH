// ======================================================
// 🧠 Runtime RemapH V3
// ------------------------------------------------------
// Ejecuta remapeos compilados.
//
// El Runtime:
//   - No lee JSON.
//   - No interpreta configuración.
//   - No conoce Windows.
//   - No conoce eventos físicos.
//
// Recibe:
//     EventoTrigger
//
// Busca:
//     Cache activa
//
// Ejecuta:
//     AccionCache
// ======================================================

use std::sync::mpsc::Sender;

use crate::cache;
use crate::evento_trigger::EventoTrigger;
use crate::perfilcache::AccionCache;

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

        println!(
            "[RUNTIME] Trigger recibido -> {:?} {:?} {:?}",
            evento.modificadores, evento.gatillo, evento.condicion
        );

        let mut activos = evento.modificadores.clone();

        activos.push(evento.gatillo.clone());

        println!("[RUNTIME] Buscando cache -> {:?}", activos);

        let Some(remapeo) = cache::buscar(&activos, &evento.gatillo) else {
            println!("[RUNTIME] Sin remapeo -> Pasar");

            return Resultado::Pasar;
        };

        println!("[RUNTIME] Remapeo encontrado");

        salida.send(remapeo.accion).unwrap();

        Resultado::Consumir
    }
}
