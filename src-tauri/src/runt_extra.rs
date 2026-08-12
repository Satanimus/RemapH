// ======================================================
// 🧩 RUNT EXTRA
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
        // 🕐 NORMAL
        // ------------------------------------------------------
        // Simula una tecla física de Windows: la primera salida
        // se dibuja una sola vez, se espera
        // config::tiempo_espera_normal(), y recién ahí arranca
        // el bucle de repetición (mismo intervalo que Turbo,
        // config::tiempo_repeticion()). INICIO_BUCLE marca dónde
        // vuelve REPETIR — así la primera espera no se repite.
        // ==================================================
        ExtraCache::Normal => vec![
            "[ACCION]".into(),
            format!("ESPERAR {}", config::tiempo_espera_normal()),
            "INICIO_BUCLE".into(),
            "[ACCION]".into(),
            format!("ESPERAR {}", config::tiempo_repeticion()),
            "REPETIR".into(),
        ],

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

        // ==================================================
        // 🔁 REPETICIÓN DE RUEDA
        // ------------------------------------------------------
        // Este brazo existe únicamente para que el `match` sea
        // exhaustivo (Rust exige cubrir todas las variantes de
        // ExtraCache, sin importar si son alcanzables en runtime).
        // Bajo el diseño elegido (Opción B, ver
        // PLAN_RUEDA_REPETICION.md), `cache::worker_repeticion_rueda`
        // maneja la cola/orden/timing por su cuenta y manda
        // `extra: None` a `runtime::ejecutar` — por lo tanto
        // `runt_extra::obtener()` NUNCA debería recibir
        // `ExtraCache::RepeticionRueda` en la práctica. Si esto
        // llegara a dispararse, es un bug de otra parte del código
        // (alguien volvió a mandar `Some(ExtraCache::RepeticionRueda)`
        // a Runtime) — mejor un panic explícito acá que una receta
        // silenciosa e incorrecta.
        // ==================================================
        ExtraCache::RepeticionRueda => {
            unreachable!(
                "runt_extra::obtener() no debería recibir RepeticionRueda: \
                 la cola la maneja cache::worker_repeticion_rueda con extra: None (Opción B)"
            )
        }
    }
}
