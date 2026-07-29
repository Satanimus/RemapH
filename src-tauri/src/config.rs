// ======================================================
// ⚙️ Config RemapH V3
// ======================================================
// ETAPA 0 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Mantiene la configuración global utilizada
// por todos los módulos del sistema.
//
// Contiene únicamente valores compartidos.
//
// NO almacena:
//
// - Perfiles.
// - Remapeos.
// - Estado.
// - Runtime.
// - UI.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe cambios desde:
//
// • Configuración persistente.
// • UI mediante comandos Tauri.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Cualquier módulo que necesite un valor
// de configuración global.
//
// Principalmente:
//
// • AnalizadorTrigger.
// • Runtime.
// • Backends.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Devuelve:
//
// • Tiempos.
// • Sensibilidades.
// • Parámetros globales.
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// tiempo_doble()
// establecer_tiempo_doble()
//     Tiempo máximo para detectar Doble.
//     También define cuánto espera AnalizadorTrigger antes de decidir que una secuencia terminó.
//
// tiempo_mantenido()
// establecer_tiempo_mantenido()
//     Tiempo mínimo para detectar Mantenido.
//
// tiempo_repeticion()
// establecer_tiempo_repeticion()
//     Intervalo entre repeticiones de Turbo.
//
// sensibilidad_rueda()
// establecer_sensibilidad_rueda()
//     Cantidad de movimientos minimos necesarios para considerar una acción mantenida.
// ------------------------------------------------------
// Transformación:
//
// UI
//      ↓
// Configuración
//      ↓
// Config
//      ↓
// Todos los módulos
// ======================================================

use std::sync::atomic::{AtomicU64, Ordering};

// ======================================================
// 📦 APP
// ======================================================

pub const NOMBRE_APP: &str = "RemapH V3";

// ======================================================
// ⏱️ TIEMPO DOBLE
// ======================================================

static TIEMPO_DOBLE: AtomicU64 = AtomicU64::new(250);

// ======================================================
// ⏳ TIEMPO MANTENIDO
// ======================================================

static TIEMPO_MANTENIDO: AtomicU64 = AtomicU64::new(300);

// ======================================================
// 🖱️ SENSIBILIDAD RUEDA
// ======================================================

static SENSIBILIDAD_RUEDA: AtomicU64 = AtomicU64::new(5);

// ======================================================
// 📥 LEER TIEMPO DOBLE
// ======================================================

pub fn tiempo_doble() -> u64 {
    TIEMPO_DOBLE.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR TIEMPO DOBLE
// ======================================================

pub fn establecer_tiempo_doble(valor: u64) {
    TIEMPO_DOBLE.store(valor, Ordering::Relaxed);
}

// ======================================================
// 📥 LEER TIEMPO MANTENIDO
// ======================================================

pub fn tiempo_mantenido() -> u64 {
    TIEMPO_MANTENIDO.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR TIEMPO MANTENIDO
// ======================================================

pub fn establecer_tiempo_mantenido(valor: u64) {
    TIEMPO_MANTENIDO.store(valor, Ordering::Relaxed);
}

// ======================================================
// 📥 LEER SENSIBILIDAD RUEDA
// ======================================================

pub fn sensibilidad_rueda() -> u64 {
    SENSIBILIDAD_RUEDA.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR SENSIBILIDAD RUEDA
// ======================================================

pub fn establecer_sensibilidad_rueda(valor: u64) {
    SENSIBILIDAD_RUEDA.store(valor, Ordering::Relaxed);
}

// ======================================================
// ⚡ TIEMPO REPETICIÓN
// ======================================================

static TIEMPO_REPETICION: AtomicU64 = AtomicU64::new(30);

// ======================================================
// 📥 LEER TIEMPO REPETICIÓN
// ======================================================

pub fn tiempo_repeticion() -> u64 {
    TIEMPO_REPETICION.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR TIEMPO REPETICIÓN
// ======================================================

pub fn establecer_tiempo_repeticion(valor: u64) {
    TIEMPO_REPETICION.store(valor, Ordering::Relaxed);
}
