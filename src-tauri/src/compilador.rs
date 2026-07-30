// ======================================================
// 🔨 COMPILADOR RemapH V3
// ======================================================
// ETAPA X DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Convierte un perfil editable (perfil_json)
// en un perfil optimizado para Runtime (perfil_cache).
//
// No ejecuta.
// No conoce dispositivos físicos.
// No conoce Runtime.
//
// Su responsabilidad:
// • Convertir estructuras.
// • Preparar triggers para búsqueda rápida.
// • Convertir respuestas de usuario en AccionCache.
//
// Flujo:
//
// perfil_json
//      ↓
// Compilador
//      ↓
// perfil_cache
//      ↓
// Cache
//      ↓
// Runtime
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe:
//
// perfil_json
//
// Contiene:
// • Remapeos.
// • Trigger.
// • Respuesta.
//
// Solo compila filas activas.
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Sistema de activación de perfil.
//
// Utilizado por:
// • Cache.
// • Sistema de perfiles.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Entrega:
//
// Vec<RemapeoCache>
//
// Ejemplo:
//
// Trigger usuario:
//
// CTRL + A
// Doble
// Firefox
//
// ↓
//
// TriggerCache:
//
// app
// entrada
// condicion
//
// Acción usuario:
//
// Tecla B
//
// ↓
//
// AccionCache:
//
// Emitir keyboard:B
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// compilar()
//     Compila perfil completo y reemplaza Cache.
//
// compilar_perfil()
//     Convierte todas las filas activas.
//
// compilar_remapeo()
//     Convierte una fila completa.
//
// convertir_app()
//     Convierte AppJson → AppCache.
//
// convertir_input()
//     Convierte Input → InputId.
//
// convertir_accion()
//     Convierte RespuestaJson → AccionCache.
// ------------------------------------------------------

use crate::cache;

use crate::eventos::InputId;

use crate::perfil_cache::{AccionCache, AppCache, ExtraCache, RemapeoCache, TriggerCache};

use crate::perfil_json::{perfil_json, AppJson, RemapeoJson};

// ======================================================
// ⚙️ COMPILAR
// ======================================================

pub fn compilar(perfil: &perfil_json) {
    let remapeos = compilar_perfil(perfil);

    cache::reemplazar(remapeos);
}

// ======================================================
// 📦 COMPILAR PERFIL
// ======================================================

pub fn compilar_perfil(perfil: &perfil_json) -> Vec<RemapeoCache> {
    perfil
        .remapeos
        .iter()
        .filter_map(compilar_remapeo)
        .collect()
}

// ======================================================
// 🧩 COMPILAR REMAPEO
// ======================================================

fn compilar_remapeo(remapeo: &RemapeoJson) -> Option<RemapeoCache> {
    if remapeo.estado != "ON" {
        return None;
    }

    Some(RemapeoCache {
        id: remapeo.id.clone(),

        trigger: TriggerCache {
            app: convertir_app(&remapeo.trigger.app),

            entrada: remapeo
                .trigger
                .entrada
                .iter()
                .map(convertir_input)
                .collect(),

            condicion: remapeo.trigger.condicion.clone(),
        },

        accion: convertir_accion(&remapeo.respuesta),

        extra: convertir_extra(&remapeo.respuesta.extra),
    })
}

// ======================================================
// 🖥️ CONVERTIR APP
// ======================================================

fn convertir_app(app: &AppJson) -> AppCache {
    match &app.programa {
        None => AppCache::Global,

        Some(nombre) => AppCache::Programa {
            nombre: nombre.clone(),

            segundo_plano: app.segundo_plano,
        },
    }
}

// ======================================================
// 🆔 CONVERTIR INPUT
// ======================================================

fn convertir_input(input: &crate::perfil_json::Input) -> InputId {
    InputId::new(&input.fuente, &input.control)
}

// ======================================================
// ⚡ CONVERTIR ACCIÓN
// ======================================================

fn convertir_accion(respuesta: &crate::perfil_json::RespuestaJson) -> AccionCache {
    match respuesta.tipo.as_str() {
        "teclado" => AccionCache::Emitir(InputId::new("keyboard", &respuesta.accion)),

        "mouse" => AccionCache::Emitir(InputId::new("mouse", &respuesta.accion)),

        "macro" => AccionCache::Macro(respuesta.accion.clone()),

        "archivo" => AccionCache::AbrirArchivo(respuesta.accion.clone()),

        "ui" => AccionCache::Ui(respuesta.accion.clone()),

        _ => {
            panic!("Acción no soportada: {}", respuesta.tipo);
        }
    }
}

// ======================================================
// 🧩 CONVERTIR EXTRA
// ======================================================

fn convertir_extra(extra: &str) -> Option<ExtraCache> {
    match extra {
        "" => None,

        "turbo" => Some(ExtraCache::Turbo),

        "mantener" => Some(ExtraCache::Mantener),

        "toggle" => Some(ExtraCache::Toggle),

        _ => None,
    }
}
