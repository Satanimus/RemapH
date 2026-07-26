// ======================================================
// 🗃️ Cache RemapH V3
// ======================================================

use crate::eventos::InputId;
use crate::perfilcache::{CondicionTrigger, RemapeoCache};

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
// 📦 ACCESO
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
// 🎯 TRIGGER EXACTO
// ======================================================

pub fn buscar(activos: &[InputId], gatillo: &InputId) -> Option<RemapeoCache> {
    let cache = obtener_cache_activa().lock().unwrap();

    cache
        .iter()
        .find(|r| {
            if r.trigger.gatillo != *gatillo {
                return false;
            }

            if activos.len() != r.trigger.modificadores.len() + 1 {
                return false;
            }

            &activos[..r.trigger.modificadores.len()] == r.trigger.modificadores.as_slice()
        })
        .cloned()
}

// ======================================================
// 🔎 EXISTE PREFIJO
// ======================================================

pub fn existe_prefijo(activos: &[InputId]) -> bool {
    let cache = obtener_cache_activa().lock().unwrap();

    cache.iter().any(|r| {
        let mut esperado = r.trigger.modificadores.clone();
        esperado.push(r.trigger.gatillo.clone());

        esperado.starts_with(activos)
    })
}

// ======================================================
// ⏳ PUEDE EXISTIR DOBLE/MANTENIDO
// ======================================================

pub fn tiene_condiciones_posibles(gatillo: &InputId) -> bool {
    let cache = obtener_cache_activa().lock().unwrap();

    cache.iter().any(|r| {
        r.trigger.gatillo == *gatillo
            && matches!(
                r.trigger.condicion,
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
        .any(|r| r.trigger.modificadores.contains(input))
}

// ======================================================
// 🎯 BUSCAR CANDIDATOS
// ------------------------------------------------------
// Devuelve todos los remapeos compatibles con:
//
// modificadores + gatillo
//
// Se utiliza para:
//
// - Captura.
// - Analizador.
// - Runtime.
//
// Luego el Analizador decidirá si corresponde:
//
// Simple
// Doble
// Mantenido
// ======================================================

pub fn buscar_candidatos(activos: &[InputId], gatillo: &InputId) -> Vec<RemapeoCache> {
    let cache = obtener_cache_activa().lock().unwrap();

    cache
        .iter()
        .filter(|r| {
            r.trigger.gatillo == *gatillo
                && activos.len() == r.trigger.modificadores.len() + 1
                && &activos[..r.trigger.modificadores.len()] == r.trigger.modificadores.as_slice()
        })
        .cloned()
        .collect()
}

// ======================================================
// ❓ EXISTE CANDIDATO
// ------------------------------------------------------
// Sólo indica si existe alguna combinación posible.
//
// No importa todavía si será:
//
// - Simple
// - Doble
// - Mantenido
// ======================================================

pub fn existe_candidato(activos: &[InputId], gatillo: &InputId) -> bool {
    !buscar_candidatos(activos, gatillo).is_empty()
}

// ======================================================
// 🖥️ CONTEXTO
// ======================================================

pub fn actualizar_contexto(_programa_activo: Option<&str>, _procesos_activos: &HashSet<String>) {
    let cache = obtener_cache().lock().unwrap();

    *obtener_cache_activa().lock().unwrap() = cache.clone();
}
