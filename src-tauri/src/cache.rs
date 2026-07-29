// ======================================================
// 🗃️ CACHE RemapH V3
// ======================================================
// ETAPA 2 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Mantiene en memoria:
//
// - Remapeos activos compilados.
// - Estado actual de aplicaciones.
//
// Cache NO conoce:
//
// - Runtime.
// - Captura.
// - Condiciones de trigger.
//
// Su responsabilidad es resolver una entrada:
//
// • Sin coincidencia.
// • Match único.
// • Necesita análisis de condición.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe:
//
// - Remapeos desde compilador.
// - Escritura o eliminación de filas.
// - Estado de aplicaciones desde back_app.
// - Entradas analizadas desde AnalizadorTrigger.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// - Compilador.
// - AnalizadorTrigger.
// - Módulo estado aplicaciones.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Devuelve ResolucionEntrada:
//
// Pasar:
//     La entrada física continúa. ***** (Pendiente definir módulo encargado de liberar)
//
// Iniciar:
//     Envía una orden completa a Runtime.
// Detener:
//     Envía señal de liberación de instancia.
//
// AnalizarCondicion:
//     Solicita al AnalizadorTrigger determinar
//     Simple / Doble / Mantenido.
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// escribir_cache()
//     Reemplaza toda la cache.
//
// escribir_fila()
//     Agrega un remapeo.
//
// borrar_cache()
//     Elimina todos los remapeos.
//
// borrar_fila()
//     Elimina un remapeo por ID.
//
// actualizar_estado_app()
//     Actualiza estado de aplicación.
//
// resolver_entrada()
//     Compara una entrada contra la cache.
//
//     Reglas:
//
//     posibles = 0
//          → Pasar.
//
//     posibles = 1
//     exactas = 1
//          → Ejecutar acción.
//
//     posibles = exactas
//     exactas > 1
//          → Analizar condición.
//
// ------------------------------------------------------
// Transformación:
//
// perfil_cache
//       ↓
//    Cache
//       ↓
// Resolución de entrada
//       ↓
// Runtime
// ======================================================

use crate::eventos::{InputEvent, InputId};

use crate::perfil_cache::{AccionCache, AppCache, ExtraCache, RemapeoCache};

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
// 📤 ORDEN RUNTIME
// ======================================================

#[derive(Clone, Debug)]
pub enum OrdenRuntime {
    Iniciar {
        id: String,

        accion: AccionCache,

        extra: Option<ExtraCache>,
    },

    Detener {
        id: String,
    },
}

// ======================================================
// 📤 RESOLUCIÓN
// ======================================================

#[derive(Clone, Debug)]
pub enum ResolucionEntrada {
    Pasar(Vec<InputEvent>),

    Ejecutar(OrdenRuntime),

    AnalizarCondicion,
}

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
// 🔎 RESOLVER ENTRADA
// ======================================================

pub fn resolver_entrada(entrada: &[InputId], eventos: Vec<InputEvent>) -> ResolucionEntrada {
    let cache = obtener_cache().lock().unwrap();

    let apps = obtener_apps().lock().unwrap();

    let mut posibles = 0;

    let mut exactas = 0;

    let mut remapeo: Option<RemapeoCache> = None;

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

    if posibles == 0 {
        return ResolucionEntrada::Pasar(eventos);
    }

    if posibles == 1 && exactas == 1 {
        let remapeo = remapeo.unwrap();

        return ResolucionEntrada::Ejecutar(OrdenRuntime::Iniciar {
            id: remapeo.id,

            accion: remapeo.accion,

            extra: remapeo.extra,
        });
    }

    if posibles == exactas && exactas > 1 {
        return ResolucionEntrada::AnalizarCondicion;
    }

    ResolucionEntrada::AnalizarCondicion
}

// ======================================================
// 🖥️ ACTUALIZAR ESTADO APP
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
