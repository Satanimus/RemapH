// ======================================================
// 🗃️ cache RemapH V3
// ======================================================
// ETAPA 2 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Mantiene en memoria todos los remapeos activos compilados y el estado de las aplicaciones.
// cache NO interpreta condiciones. NO decide la respuesta. NO conoce Runtime.
//
// Su única responsabilidad es responder:
// • ¿Existe alguna posible coincidencia?
// • ¿Existe una coincidencia exacta?
//
// Flujo:
// perfil_json
//     ↓
// compilador
//     ↓
// perfil_cache
//     ↓
// cache
// ------------------------------------------------------
// 2. ¿Qué información recibe?
// Recibe:
// • Remapeos compilados.
// • Escritura o eliminación de filas.
// • Actualización del estado de aplicaciones desde Windows.
// • Entradas del Analizador.
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
// Recibe llamadas desde:
// • compilador  • analizador_trigger • módulo de estado de aplicaciones
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// Devuelve únicamente:
// • Cantidad de coincidencias posibles.
// • Cantidad de coincidencias exactas.
// • El remapeo completo únicamente cuando existe un MATCH único.
//
// Nunca interpreta:   • Simple • Doble • Mantenido
// Esa responsabilidad pertenece al Analizador Trigger.
// ------------------------------------------------------
// 5. Funciones del archivo
// escribir_cache()
//     Reemplaza completamente la cache.
// escribir_fila()
//     Agrega una fila.
// borrar_cache()
//     Elimina toda la cache.
// borrar_fila()
//     Elimina una fila.
// actualizar_estado_app()
//     Actualiza el estado de una aplicación.
// buscar()
//     Compara la entrada contra la cache.
// ------------------------------------------------------
// Transformación:
// perfil_cache
//        ↓
//     cache
//        ↓
// Analizador Trigger
// ======================================================

use crate::eventos::InputId;
use crate::perfil_cache::{AppCache, RemapeoCache};

use std::sync::{Mutex, OnceLock};

// ======================================================
// 📦 CACHE REMAPEOS
// ======================================================

static CACHE: OnceLock<Mutex<Vec<RemapeoCache>>> = OnceLock::new();

// ======================================================
// 🖥️ ESTADO APLICACIONES
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub struct AppEstadoCache {
    pub app: AppCache,

    pub activa: bool,
}

static APPS: OnceLock<Mutex<Vec<AppEstadoCache>>> = OnceLock::new();

// ======================================================
// 📦 ACCESO CACHE
// ======================================================

fn obtener_cache() -> &'static Mutex<Vec<RemapeoCache>> {
    CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

// ======================================================
// 🖥️ ACCESO APPS
// ======================================================

fn obtener_apps() -> &'static Mutex<Vec<AppEstadoCache>> {
    APPS.get_or_init(|| Mutex::new(Vec::new()))
}

// ======================================================
// ✍️ ESCRIBIR CACHE
// ======================================================

pub fn escribir_cache(remapeos: Vec<RemapeoCache>) {
    *obtener_cache().lock().unwrap() = remapeos;
}

// ======================================================
// ✍️ ESCRIBIR FILA
// ======================================================

pub fn escribir_fila(remapeo: RemapeoCache) {
    obtener_cache().lock().unwrap().push(remapeo);
}

// ======================================================
// 🗑️ BORRAR CACHE
// ======================================================

pub fn borrar_cache() {
    obtener_cache().lock().unwrap().clear();
}

// ======================================================
// 🗑️ BORRAR FILA
// ======================================================

pub fn borrar_fila(id: &str) {
    obtener_cache().lock().unwrap().retain(|r| r.id != id);
}

// ======================================================
// 🔎 RESULTADO BÚSQUEDA
// ======================================================

#[derive(Clone, Debug)]
pub struct ResultadoBusqueda {
    pub posibles: usize,

    pub exactas: usize,

    pub remapeo: Option<RemapeoCache>,
}

// ======================================================
// 🔎 BUSCAR
// ------------------------------------------------------
// Compara la entrada recibida contra todos los
// remapeos compatibles con el contexto actual.
//
// Devuelve:
//
// posibles
// exactas
// remapeo (solo si existe un MATCH único)
// ======================================================

pub fn buscar(entrada: &[InputId]) -> ResultadoBusqueda {
    let cache = obtener_cache().lock().unwrap();

    let apps = obtener_apps().lock().unwrap();

    let mut posibles = 0;

    let mut exactas = 0;

    let mut remapeo = None;

    for fila in cache.iter() {
        if !app_habilitada(&fila.trigger.app, &apps) {
            continue;
        }

        if fila.trigger.entrada.starts_with(entrada) {
            posibles += 1;
        }

        if fila.trigger.entrada == entrada {
            exactas += 1;

            if exactas == 1 {
                remapeo = Some(fila.clone());
            } else {
                remapeo = None;
            }
        }
    }

    ResultadoBusqueda {
        posibles,

        exactas,

        remapeo,
    }
}

// ======================================================
// 🖥️ ACTUALIZAR ESTADO APP
// ------------------------------------------------------
// Windows informará:
//
// - abre
// - cierra
// - gana foco
// - pierde foco
//
// Cache solamente actualiza el estado.
// ======================================================

pub fn actualizar_estado_app(app: &AppCache, activa: bool) {
    let mut apps = obtener_apps().lock().unwrap();

    if let Some(actual) = apps.iter_mut().find(|a| a.app == *app) {
        actual.activa = activa;

        return;
    }

    apps.push(AppEstadoCache {
        app: app.clone(),

        activa,
    });
}

// ======================================================
// ❓ APP HABILITADA
// ------------------------------------------------------
// Determina si un remapeo debe participar
// en la búsqueda.
//
// Global siempre participa.
//
// Programa depende del estado almacenado
// previamente por Windows.
// ======================================================

fn app_habilitada(app: &AppCache, estados: &[AppEstadoCache]) -> bool {
    match app {
        AppCache::Global => true,

        _ => estados
            .iter()
            .find(|estado| estado.app == *app)
            .map(|estado| estado.activa)
            .unwrap_or(false),
    }
}
