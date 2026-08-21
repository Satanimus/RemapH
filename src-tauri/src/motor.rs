// ======================================================
// ⚙️ MOTOR
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Punto único de despacho entre los dos backends de
// entrada/salida que puede tener RemapH: back_interception
// (driver Interception) y back_windows (hooks WinAPI, Modo
// Portable — ver REGLAS_MODO_PORTABLE.txt).
//
// Nadie más que este archivo debería llamar directo a
// back_interception::iniciar()/emitir_evento() ni a
// back_windows::iniciar()/emitir_evento() — todo el resto
// del proyecto (lib.rs, entrada.rs, runtime.rs) pasa por
// motor::iniciar()/motor::emitir_evento(), que decide cuál
// de los dos backends usar según el modo activo.
//
// No traduce ni interpreta eventos. No conoce Runtime,
// Cache ni AnalizadorTrigger — solo reenvía.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// lib.rs llama motor::iniciar() una sola vez al arrancar,
// dentro del hilo de entrada.
// entrada.rs / Runtime llaman motor::emitir_evento() en
// vez de llamar directo a back_interception::emitir_evento()
// o back_windows::emitir_evento().
// El futuro flujo de cambio de modo (Etapa D) llama
// motor::establecer_modo().
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// iniciar(procesar, debe_tragar_no_traducible): misma firma
// que back_interception::iniciar()/back_windows::iniciar().
// emitir_evento(evento): un InputEvent completo.
// establecer_modo(modo): un Modo (Interception o Portable).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// modo_activo() -> Modo: el modo actualmente activo.
// El resto no retorna valor.
// ------------------------------------------------------
// 5. Funciones del archivo
// modo_activo() / establecer_modo(modo)
//     Getter/setter del modo activo en memoria (Modo::
//     Interception por defecto). Solo el valor — sin
//     persistir a disco (Etapa C) ni cortar/arrancar
//     backends en caliente (Etapa D).
// iniciar(procesar, debe_tragar_no_traducible)
//     Según modo_activo(), llama a back_interception::
//     iniciar() (con precargar_desde_config() antes) o a
//     back_windows::iniciar().
// emitir_evento(evento)
//     Según modo_activo(), despacha a back_interception::
//     emitir_evento() o back_windows::emitir_evento().
// ======================================================

use std::sync::atomic::{AtomicU8, Ordering};

use crate::eventos::InputEvent;
use crate::{back_interception, back_windows};

// ======================================================
// 🔀 MODO
// ------------------------------------------------------
// Interception = 0, Portable = 1. Interception es el valor
// por defecto (arranque en frío, antes de que Etapa C cargue
// el modo guardado en Configuracion_Usuario.txt).
// ======================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Modo {
    Interception,
    Portable,
}

static MODO_ACTIVO: AtomicU8 = AtomicU8::new(0);

pub fn modo_activo() -> Modo {
    match MODO_ACTIVO.load(Ordering::SeqCst) {
        1 => Modo::Portable,
        _ => Modo::Interception,
    }
}

// (Nota: solo cambia el valor en memoria — sin persistir a
// disco (Etapa C) ni disparar el corte/arranque de backends
// en caliente (Etapa D). Acá es un simple getter/setter.)
pub fn establecer_modo(modo: Modo) {
    let valor = match modo {
        Modo::Interception => 0,
        Modo::Portable => 1,
    };

    MODO_ACTIVO.store(valor, Ordering::SeqCst);
}

// ======================================================
// 🚀 INICIAR
// ------------------------------------------------------
// La firma pide `+ 'static` en ambos parámetros porque es
// el requisito más estricto de los dos backends —
// back_windows::iniciar() los necesita 'static porque los
// guarda en su ESTADO por thread_local (Box<dyn ...>);
// back_interception::iniciar() no exige 'static, así que un
// valor 'static igual lo satisface sin problema.
//
// Solo una de las dos ramas se ejecuta en cada llamada — no
// hay problema de mover procesar/debe_tragar_no_traducible
// en los dos brazos del match, el compilador rastrea el
// move por rama, no exige que sea el mismo en ambas.
// ======================================================

pub fn iniciar(
    procesar: impl FnMut(InputEvent) + 'static,
    debe_tragar_no_traducible: impl Fn() -> bool + 'static,
) {
    match modo_activo() {
        Modo::Interception => {
            back_interception::precargar_desde_config();
            back_interception::iniciar(procesar, debe_tragar_no_traducible);
        }

        Modo::Portable => {
            back_windows::iniciar(procesar, debe_tragar_no_traducible);
        }
    }
}

// ======================================================
// 📤 EMITIR EVENTO
// ======================================================

pub fn emitir_evento(evento: InputEvent) {
    match modo_activo() {
        Modo::Interception => back_interception::emitir_evento(evento),
        Modo::Portable => back_windows::emitir_evento(evento),
    }
}
