// ======================================================
// 🧩 RUNT EXTRA RemapH V3
// ======================================================
// ETAPA 8 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Mantiene una biblioteca de comportamientos
// prediseñados.
//
// NO:
//
// - Ejecuta acciones.
// - Conoce Runtime.
// - Conoce Cache.
// - Conoce dispositivos.
//
// Su única responsabilidad es transformar
// un ExtraCache en una receta de macro.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe:
//
// • ExtraCache.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// • Runtime.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Devuelve:
//
// Vec<String>
//
// Cada elemento corresponde a una instrucción
// del lenguaje de macros.
//
// Runtime reemplazará posteriormente:
//
// • [ACCION]
// • [ACCION_DOWN]
// • [ACCION_UP]
//
// y ejecutará la receta resultante.
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// obtener()
//     Devuelve la receta correspondiente
//     al Extra solicitado.
//
// ------------------------------------------------------
// Transformación:
//
// ExtraCache
//      ↓
// Runt Extra
//      ↓
// Vec<String>
//      ↓
// Runtime
//      ↓
// Ejecutor Macro
// ======================================================

use crate::config;
use crate::perfil_cache::ExtraCache;

// ======================================================
// 📜 OBTENER RECETA
// ======================================================

pub fn obtener(extra: &ExtraCache) -> Vec<String> {
    match extra {
        // ==================================================
        // ⚡ TURBO
        // ==================================================
        ExtraCache::Turbo => vec![
            "[ACCION]".into(),
            format!("ESPERAR {}", config::tiempo_repeticion()),
            "REPETIR".into(),
        ],

        // ==================================================
        // 🔒 MANTENER
        // ==================================================
        ExtraCache::Mantener => vec![
            "[ACCION_DOWN]".into(),
            "ESPERAR DETENER".into(),
            "[ACCION_UP]".into(),
        ],

        // ==================================================
        // 🔀 TOGGLE
        // ==================================================
        ExtraCache::Toggle => vec!["TOGGLE [ACCION]".into()],

        // ==================================================
        // 🖱️ DOBLE CLICK
        // ==================================================
        ExtraCache::DobleClick => vec!["[ACCION]".into(), "ESPERAR 50".into(), "[ACCION]".into()],

        // ==================================================
        // 🖱️ CLICK SOSTENIDO
        // ==================================================
        ExtraCache::ClickSostenido => vec![
            "[ACCION_DOWN]".into(),
            "ESPERAR DETENER".into(),
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
