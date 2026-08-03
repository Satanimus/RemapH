// ======================================================
// ⏱️ INSTANTE
// ======================================================
// ETAPA 0 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Genera una referencia temporal única para todo el motor.
// No mide duraciones. /  No calcula tiempos. / No interpreta eventos.
// Su única responsabilidad es responder:
// "¿Qué hora interna tiene el motor en este momento?"
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// Backend de captura.
// Actualmente:
// - back_interception ¿?
// Futuro:
// - back_entrada (Full)
// - cualquier backend físico de entrada.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// No recibe información.
// Simplemente consulta el reloj interno del programa.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// Instante (u64)
// Unidad: milisegundos desde el inicio del programa.
// Ejemplo:
// 1523
// Significa:
// "Han transcurrido 1523 ms desde que RemapH inició."
// ------------------------------------------------------
// 5. Funciones del archivo
// inicio()
//     Inicializa el reloj base del programa.
//     Se ejecuta una sola vez.
// ahora()
//     Devuelve el instante actual del motor.
// ------------------------------------------------------
// Transformación que realiza
// Inicio del programa
//      ↓
// reloj interno = 0 ms
//      ↓
// Evento físico
//      ↓
// ahora()
//      ↓
// 1523
//      ↓
// Ese valor acompañará al InputEvent durante todo
// el recorrido del sistema.
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
