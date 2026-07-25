// ======================================================
// 🗃️ Cache RemapH V3
// ======================================================

use crate::eventos::InputId;
use crate::perfilcache::{AccionCache, AppCache, CondicionTrigger, RemapeoCache};

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

// ======================================================
// 📦 CACHE
// ======================================================

static CACHE: OnceLock<Mutex<Vec<RemapeoCache>>> = OnceLock::new();

static CACHE_ACTIVA: OnceLock<Mutex<Vec<RemapeoCache>>> = OnceLock::new();

// ======================================================
// 🔒 TESTS
// ======================================================

#[cfg(test)]
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(test)]
pub fn bloquear_tests() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

// ======================================================
// 📦 OBTENER CACHE
// ======================================================

fn obtener_cache() -> &'static Mutex<Vec<RemapeoCache>> {
    CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

fn obtener_cache_activa() -> &'static Mutex<Vec<RemapeoCache>> {
    CACHE_ACTIVA.get_or_init(|| Mutex::new(Vec::new()))
}

// ======================================================
// 🔄 REEMPLAZAR
// ======================================================

pub fn reemplazar(remapeos: Vec<RemapeoCache>) {
    *obtener_cache().lock().unwrap() = remapeos.clone();

    *obtener_cache_activa().lock().unwrap() = remapeos;
}

// ======================================================
// 🗑️ BORRAR
// ======================================================

pub fn borrar() {
    reemplazar(Vec::new());
}

// ======================================================
// ❓ VACÍA
// ======================================================

pub fn esta_vacia() -> bool {
    obtener_cache().lock().unwrap().is_empty()
}

// ======================================================
// 🎯 BUSCAR TRIGGER EXACTO
// ======================================================

pub fn buscar(activos: &[InputId], gatillo: &InputId) -> Option<RemapeoCache> {
    let cache = obtener_cache_activa().lock().unwrap();

    cache
        .iter()
        .find(|remapeo| {
            if remapeo.trigger.gatillo != *gatillo {
                return false;
            }

            let modificadores = &remapeo.trigger.modificadores;

            if activos.len() != modificadores.len() + 1 {
                return false;
            }

            &activos[..modificadores.len()] == modificadores.as_slice()
        })
        .cloned()
}

// ======================================================
// ⏳ PUEDE CONTINUAR
// ------------------------------------------------------
// Determina si la secuencia actual puede formar
// algún trigger futuro.
// ======================================================

pub fn puede_continuar(activos: &[InputId]) -> bool {
    let cache = obtener_cache_activa().lock().unwrap();

    cache.iter().any(|remapeo| {
        let mut esperado = remapeo.trigger.modificadores.clone();

        esperado.push(remapeo.trigger.gatillo.clone());

        esperado.starts_with(activos)
    })
}

// ======================================================
// 🔎 EXISTEN CONDICIONES FUTURAS
// ======================================================

pub fn tiene_condiciones_posibles(gatillo: &InputId) -> bool {
    let cache = obtener_cache_activa().lock().unwrap();

    cache.iter().any(|remapeo| {
        remapeo.trigger.gatillo == *gatillo
            && matches!(
                remapeo.trigger.condicion,
                CondicionTrigger::Doble | CondicionTrigger::Mantenido
            )
    })
}

// ======================================================
// 🔎 ES MODIFICADOR
// ======================================================

pub fn es_modificador(input: &InputId) -> bool {
    let cache = obtener_cache_activa().lock().unwrap();

    cache
        .iter()
        .any(|remapeo| remapeo.trigger.modificadores.contains(input))
}

// ======================================================
// 🖥️ CONTEXTO APP
// ======================================================

pub fn actualizar_contexto(programa_activo: Option<&str>, procesos_activos: &HashSet<String>) {
    let cache = obtener_cache().lock().unwrap();

    let activa = cache.iter().filter(|_| true).cloned().collect();

    drop(cache);

    *obtener_cache_activa().lock().unwrap() = activa;
}
