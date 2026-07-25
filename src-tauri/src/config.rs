// ======================================================
// ⚙️ Config RemapH V3
// ------------------------------------------------------
// Valores compartidos por todo el sistema.
//
// UI
// ↓
// Configuración
// ↓
// Captura / Analizador / Runtime
// ======================================================

use std::sync::atomic::{AtomicU64, Ordering};

// ======================================================
// 📦 APP
// ======================================================

pub const NOMBRE_APP: &str = "RemapH V3";

// ======================================================
// ⏱️ TIEMPO DOBLE
// ------------------------------------------------------
// Tiempo máximo entre pulsaciones para considerar doble.
//
// También define cuánto espera AnalizadorTrigger
// antes de decidir que una secuencia terminó.
// ======================================================

static TIEMPO_DOBLE: AtomicU64 = AtomicU64::new(250);

// ======================================================
// ⏳ TIEMPO MANTENIDO
// ------------------------------------------------------
// Tiempo mínimo presionado para considerar mantenido.
// ======================================================

static TIEMPO_MANTENIDO: AtomicU64 = AtomicU64::new(300);

// ======================================================
// 🖱️ SENSIBILIDAD RUEDA
// ------------------------------------------------------
// Cantidad de movimientos necesarios para considerar
// una acción mantenida de rueda.
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
