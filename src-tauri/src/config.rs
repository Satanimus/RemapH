// ======================================================
// ⚙️ Config RemapH V3
// ------------------------------------------------------
// Valores compartidos por todo el sistema.
// ======================================================

use std::sync::atomic::{AtomicU64, Ordering};

// ======================================================
// 📦 APP
// ======================================================

pub const NOMBRE_APP: &str = "RemapH V3";

// ======================================================
// ⏱️ TIEMPO DOBLE
// ------------------------------------------------------
// También define cuánto espera el AnalizadorTrigger
// antes de decidir que una secuencia terminó.
// ======================================================

static TIEMPO_DOBLE: AtomicU64 = AtomicU64::new(250);

// ======================================================
// 📥 LEER
// ======================================================

pub fn tiempo_doble() -> u64 {
    TIEMPO_DOBLE.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR
// ======================================================

pub fn establecer_tiempo_doble(valor: u64) {
    TIEMPO_DOBLE.store(valor, Ordering::Relaxed);
}
