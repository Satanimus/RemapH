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
//     Ademas ejecuta orden revisar_app() para conocer si Apps del filtro están activas
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
// convertir_entrada()
//     Aplana modificadores + gatillo de un TriggerJson
//     en un solo Vec<InputId>.
//
// convertir_accion()
//     Resuelve accion_trigger/accion_referencia según
//     tipo → Option<AccionCache>. None (tipo decorativo
//     todavía no soportado) descarta la fila entera.
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

    cache::borrar_cache();
    cache::escribir_cache(remapeos);

    crate::back_app::revisar_apps();
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

    let accion = convertir_accion(remapeo)?;

    Some(RemapeoCache {
        id: remapeo.id.clone(),

        trigger: TriggerCache {
            app: convertir_app(&remapeo.app),

            entrada: convertir_entrada(&remapeo.trigger),

            condicion: remapeo.trigger.condicion.clone(),
        },

        accion,

        extra: convertir_extra(&remapeo.extra),
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
// ⌨️ CONVERTIR ENTRADA (aplanar trigger)
// ------------------------------------------------------
// modificadores + gatillo, en ese orden, a un solo
// Vec<InputId> — es lo único que Cache necesita para
// comparar (no distingue modificador de gatillo).
// ======================================================

fn convertir_entrada(trigger: &crate::perfil_json::TriggerJson) -> Vec<InputId> {
    trigger
        .modificadores
        .iter()
        .map(convertir_input)
        .chain(trigger.gatillo.iter().map(convertir_input))
        .collect()
}

// ======================================================
// ⚡ CONVERTIR ACCIÓN
// ------------------------------------------------------
// accion_trigger / accion_referencia es una caja cuyo
// contenido depende de tipo: para tecla_mouse se usa
// solo el gatillo del accion_trigger (Emitir es un único
// control, sin modificadores ni condición). Para
// macro/archivo/ui se usa accion_referencia tal cual.
// Cualquier otro tipo (decorativo, sin implementar todavía
// en Rust) hace que la fila se descarte al compilar.
// ======================================================

fn convertir_accion(remapeo: &RemapeoJson) -> Option<AccionCache> {
    match remapeo.tipo.as_str() {
        "tecla_mouse" => {
            let gatillo = remapeo
                .accion_trigger
                .as_ref()
                .and_then(|trigger| trigger.gatillo.as_ref())
                .unwrap_or_else(|| {
                    panic!(
                        "Remapeo '{}': tipo '{}' sin gatillo en accion_trigger",
                        remapeo.id, remapeo.tipo
                    )
                });

            Some(AccionCache::Emitir(convertir_input(gatillo)))
        }

        "macro" => Some(AccionCache::Macro(referencia(remapeo))),

        "archivo" => Some(AccionCache::AbrirArchivo(referencia(remapeo))),

        "ui" => Some(AccionCache::Ui(referencia(remapeo))),

        // Tipos todavía decorativos en la UI (Multimedia, Click
        // coordenada, Portapapeles, etc.): no producen ninguna acción
        // real todavía. La fila entera se descarta al compilar (ver
        // compilar_remapeo), igual que una fila en OFF.
        _ => None,
    }
}

// ======================================================
// 📎 REFERENCIA (accion_referencia obligatoria)
// ======================================================

fn referencia(remapeo: &RemapeoJson) -> String {
    remapeo.accion_referencia.clone().unwrap_or_else(|| {
        panic!(
            "Remapeo '{}': tipo '{}' sin accion_referencia",
            remapeo.id, remapeo.tipo
        )
    })
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
