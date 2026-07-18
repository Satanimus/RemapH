// ======================================================
// 🔒 Reentrada RemapH V3
// ------------------------------------------------------
// Evita que eventos generados por el motor se procesen
// nuevamente.
// ======================================================

use std::sync::atomic::{AtomicBool, Ordering};

static BLOQUEADO: AtomicBool = AtomicBool::new(false);

pub fn esta_bloqueado() -> bool {
    BLOQUEADO.load(Ordering::SeqCst)
}

pub fn bloquear() {
    BLOQUEADO.store(true, Ordering::SeqCst);
}

pub fn liberar() {
    BLOQUEADO.store(false, Ordering::SeqCst);
}