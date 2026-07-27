// ======================================================
// 🔨 COMPILADOR RemapH V3
// ======================================================
// ETAPA X DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Convierte un perfil editable (perfil_json) en un perfil optimizado para Runtime (perfil_cache).
// No valida. No interpreta. No ejecuta.
//  Su única responsabilidad es transformar estructuras.
//
// Flujo:
// perfil_json
//      ↓
// Compilador
//      ↓
// perfil_cache
//      ↓
// Cache
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// Sistema de activación de perfil.
// Es utilizado por:
// - Cache. - Runtime.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// Recibe: perfil_json
// Contiene: - Remapeos. - Trigger. - Respuesta.
// Solo compila filas con estado ON.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// Entrega:
// Vec<RemapeoCache>
//
// Ejemplo:
// perfil_json:
//
// CTRL + A
// Doble
// Firefox
// Enviar B
// ↓
// perfil_cache:
// TriggerCache {
//    app,
//    entrada,
//    condicion }
// RespuestaCache {
//    tipo,
//    accion,
//    ejecucion }
// ------------------------------------------------------
// 5. Funciones del archivo
// compilar()
//     Compila perfil y actualiza Cache.
// compilar_perfil()
//     Convierte todas las filas activas.
// compilar_remapeo()
//     Convierte una fila.
// convertir_app()
//     Convierte AppJson → AppCache.
// convertir_input()
//     Convierte Input → InputId.
// ======================================================

use crate::cache;

use crate::eventos::InputId;

use crate::perfil_cache::{AppCache, RemapeoCache, RespuestaCache, TriggerCache};

use crate::perfil_json::{perfil_json, AppJson, RemapeoJson};

// ======================================================
// ⚙️ COMPILAR PERFIL COMPLETO
// ======================================================

pub fn compilar(perfil: &perfil_json) {
    let remapeos = compilar_perfil(perfil);

    let cantidad = remapeos.len();

    cache::reemplazar(remapeos);

    println!("🔨 {} remapeos compilados", cantidad);
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

            condicion: convertir_condicion(&remapeo.trigger.condicion),
        },

        respuesta: RespuestaCache {
            tipo: remapeo.respuesta.tipo.clone(),

            accion: remapeo.respuesta.accion.clone(),

            ejecucion: remapeo.respuesta.ejecucion.clone(),
        },
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
// 🆔 INPUT → INPUT ID
// ======================================================

fn convertir_input(input: &crate::idioma::Input) -> InputId {
    InputId::new(&input.fuente, &input.control)
}
