// ======================================================
// ⏱️ Instante RemapH V3
// ------------------------------------------------------
// Reloj interno del motor.
//
// No representa:
//   - Duración.
//   - Tiempo configurado.
//   - Delay.
//   - Mantenido.
//
// Solo representa:
//   "momento exacto en que ocurrió algo".
//
// Usado por:
//   - AnalizadorTrigger
//   - Captura
// ======================================================

use std::time::Instant as Reloj;

// ======================================================
// 🕒 ORIGEN DEL RELOJ
// ======================================================

static INICIO: std::sync::OnceLock<Reloj> = std::sync::OnceLock::new();

// ======================================================
// 🏗️ OBTENER ORIGEN
// ======================================================

fn inicio() -> &'static Reloj {
    INICIO.get_or_init(Reloj::now)
}

// ======================================================
// ⏱️ INSTANTE ACTUAL
// ======================================================
//
// Devuelve milisegundos desde el inicio
// del programa.
//
// Nunca vuelve a cero.
// No se reinicia.
// No depende de Windows.
//

pub fn ahora() -> u64 {
    inicio().elapsed().as_millis() as u64
}
