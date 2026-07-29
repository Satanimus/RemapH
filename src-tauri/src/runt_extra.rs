// ======================================================
// 🧩 RUNT EXTRA RemapH V3
// ======================================================
//
// Biblioteca de plantillas de comportamiento.
//
// Extra NO:
// - Ejecuta acciones.
// - Conoce Runtime.
// - Conoce Cache.
// - Conoce dispositivos.
//
// Su única responsabilidad:
//
// Entregar macros prediseñadas.
//
// Runtime reemplaza los marcadores:
//
// [ACCION]
// [ACCION_DOWN]
// [ACCION_UP]
//
// y luego ejecuta la macro resultante.
//
// Extra es una colección de macros
// frecuentes simplificadas.
//
// Flujo:
//
// ExtraCache
//      ↓
// runt_extra
//      ↓
// Vec<String>
//      ↓
// Runtime
//      ↓
// Ejecutor Macro
//      ↓
// Backends físicos
//
// ======================================================

// ======================================================
// 🧩 EXTRA CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub enum ExtraCache {
    Turbo,

    Mantener,

    Toggle,

    DobleClick,

    ClickSostenido,

    AbrirMinimizado,

    PopupToggle,
}

// ======================================================
// 📜 OBTENER MACRO EXTRA
// ======================================================
//
// Devuelve líneas en lenguaje Runtime.
//
// Estas líneas todavía tienen
// marcadores pendientes.
//
// ======================================================

use crate::perfil_cache::ExtraCache;

// ======================================================
// 📜 OBTENER RECETA EXTRA
// ======================================================
//
// Devuelve pasos en lenguaje Runtime.
//
// Los marcadores son reemplazados
// posteriormente por Runtime.
//
// ======================================================

pub fn obtener(extra: &ExtraCache) -> Vec<String> {
    match extra {
        // ==================================================
        // 🔥 TURBO
        //
        // Acción
        // Espera
        // Repite
        // ==================================================
        ExtraCache::Turbo => vec!["[ACCION]".into(), "WAIT 30".into(), "LOOP".into()],

        // ==================================================
        // 🔒 MANTENER
        //
        // Mantiene pulsado hasta recibir detener.
        // ==================================================
        ExtraCache::Mantener => vec![
            "[ACCION_DOWN]".into(),
            "WAIT STOP".into(),
            "[ACCION_UP]".into(),
        ],

        // ==================================================
        // 🔀 TOGGLE
        //
        // Cambia estado de la acción.
        //
        // Runtime resolverá el estado.
        // ==================================================
        ExtraCache::Toggle => vec!["TOGGLE [ACCION]".into()],

        // ==================================================
        // 🖱️ DOBLE CLICK
        // ==================================================
        ExtraCache::DobleClick => vec!["[ACCION]".into(), "WAIT 50".into(), "[ACCION]".into()],

        // ==================================================
        // 🖱️ CLICK SOSTENIDO
        // ==================================================
        ExtraCache::ClickSostenido => vec![
            "[ACCION_DOWN]".into(),
            "WAIT STOP".into(),
            "[ACCION_UP]".into(),
        ],

        // ==================================================
        // 📂 ABRIR MINIMIZADO
        // ==================================================
        ExtraCache::AbrirMinimizado => vec!["OPEN MINIMIZED".into()],

        // ==================================================
        // 🪟 POPUP TOGGLE
        // ==================================================
        ExtraCache::PopupToggle => vec!["TOGGLE POPUP".into()],
    }
}
