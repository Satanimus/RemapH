// ======================================================
// 🖱️ Back_Mouse
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Traduce entre el mouse físico (flags MouseFilter +
// rueda, de la librería interception) y el idioma
// interno del motor (InputId).
//
// El nombre de cada botón/rueda lo decide únicamente la
// columna "interception" de pulsadores.tsv. Este archivo
// no inventa nombres — solo sabe reconocer qué flag
// físico corresponde a qué nombre, y consulta el
// diccionario para confirmar el interno.
//
// No conoce AnalizadorTrigger.
// No conoce Cache.
// No conoce Runtime.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// back_interception (traducir(), para la entrada)
// back_interception, para la salida (convertir_salida())
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// convertir(): el estado de flags del mouse (MouseFilter)
// + el valor de rueda (rolling) de un Stroke.
// convertir_salida(): un InputId (interno) a emitir.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// convertir(): Option<Evento>. None si el flag no
// corresponde a nada reconocido.
// convertir_salida(): Option<MouseOutput>. None si el
// InputId no es de mouse, o no tiene control asociado.
// ------------------------------------------------------
// 5. Funciones del archivo
// MouseOutput
//     Describe qué emitir: un botón (flags Down/Up) o
//     un movimiento de rueda (magnitud con signo).
// construir()
//     Arma el Evento para un nombre de interception dado,
//     consultando pulsadores.tsv. Registra en consola los
//     controles no soportados.
// convertir()
//     Flags + rueda → Evento (vía pulsadores.tsv).
// convertir_salida()
//     InputId → MouseOutput (vía pulsadores.tsv).
// ------------------------------------------------------
// Transformación:
//
// ENTRADA:
// MouseFilter + rolling (interception)
//     ↓
// pulsadores::interception_a_interno()
//     ↓
// InputId (interno)
//     ↓
// Evento
//
// SALIDA:
// InputId (interno)
//     ↓
// pulsadores::interno_a_interception()
//     ↓
// MouseOutput (interception)
// ======================================================

use crate::eventos::{InputEvent, InputId};
use crate::pulsadores;
use interception::MouseFilter;

// ======================================================
// 📤 OUTPUT DE MOUSE
// ======================================================

pub enum MouseOutput {
    Button { down: MouseFilter, up: MouseFilter },

    Wheel(i16),
}

// ======================================================
// 🏗️ CONSTRUIR EVENTO
// ======================================================

fn construir(interception: &str, armar: impl FnOnce(InputId) -> InputEvent) -> Option<InputEvent> {
    let Some(interno) = pulsadores::interception_a_interno(interception) else {
        println!("⚠️ Control de mouse no soportado: {}", interception);

        return None;
    };

    Some(armar(InputId::new("mouse", interno)))
}

// ======================================================
// 📥 CONVERTIR ENTRADA
// ======================================================

pub fn convertir(state: MouseFilter, rolling: i16) -> Option<InputEvent> {
    // ----------------------------------------------
    // 🖱️ RUEDA
    // ----------------------------------------------

    if rolling > 0 {
        return construir("WheelUp", |input| {
            InputEvent::pulse_con_magnitud(input, rolling)
        });
    }

    if rolling < 0 {
        return construir("WheelDown", |input| {
            InputEvent::pulse_con_magnitud(input, rolling)
        });
    }

    // ----------------------------------------------
    // 🖱️ BOTONES
    // ----------------------------------------------

    if state.contains(MouseFilter::LEFT_BUTTON_DOWN) {
        return construir("LeftButton", InputEvent::down);
    }

    if state.contains(MouseFilter::LEFT_BUTTON_UP) {
        return construir("LeftButton", InputEvent::up);
    }

    if state.contains(MouseFilter::RIGHT_BUTTON_DOWN) {
        return construir("RightButton", InputEvent::down);
    }

    if state.contains(MouseFilter::RIGHT_BUTTON_UP) {
        return construir("RightButton", InputEvent::up);
    }

    if state.contains(MouseFilter::MIDDLE_BUTTON_DOWN) {
        return construir("MiddleButton", InputEvent::down);
    }

    if state.contains(MouseFilter::MIDDLE_BUTTON_UP) {
        return construir("MiddleButton", InputEvent::up);
    }

    if state.contains(MouseFilter::BUTTON_4_DOWN) {
        return construir("Button4", InputEvent::down);
    }

    if state.contains(MouseFilter::BUTTON_4_UP) {
        return construir("Button4", InputEvent::up);
    }

    if state.contains(MouseFilter::BUTTON_5_DOWN) {
        return construir("Button5", InputEvent::down);
    }

    if state.contains(MouseFilter::BUTTON_5_UP) {
        return construir("Button5", InputEvent::up);
    }

    // ----------------------------------------------
    // ❌ EVENTO NO SOPORTADO
    // ----------------------------------------------

    None
}

// ======================================================
// 📤 CONVERTIR OUTPUT
// ======================================================

pub fn convertir_salida(input: &InputId) -> Option<MouseOutput> {
    if input.fuente() != Some("mouse") {
        return None;
    }

    let interno = input.control()?;

    let interception = pulsadores::interno_a_interception(interno)?;

    match interception {
        // ----------------------------------------------
        // 🖱️ BOTONES
        // ----------------------------------------------
        "LeftButton" => Some(MouseOutput::Button {
            down: MouseFilter::LEFT_BUTTON_DOWN,

            up: MouseFilter::LEFT_BUTTON_UP,
        }),

        "RightButton" => Some(MouseOutput::Button {
            down: MouseFilter::RIGHT_BUTTON_DOWN,

            up: MouseFilter::RIGHT_BUTTON_UP,
        }),

        "MiddleButton" => Some(MouseOutput::Button {
            down: MouseFilter::MIDDLE_BUTTON_DOWN,

            up: MouseFilter::MIDDLE_BUTTON_UP,
        }),

        "Button4" => Some(MouseOutput::Button {
            down: MouseFilter::BUTTON_4_DOWN,

            up: MouseFilter::BUTTON_4_UP,
        }),

        "Button5" => Some(MouseOutput::Button {
            down: MouseFilter::BUTTON_5_DOWN,

            up: MouseFilter::BUTTON_5_UP,
        }),

        // ----------------------------------------------
        // 🖱️ RUEDA
        // ----------------------------------------------
        "WheelUp" => Some(MouseOutput::Wheel(120)),

        "WheelDown" => Some(MouseOutput::Wheel(-120)),

        // ----------------------------------------------
        // ❌ NO SOPORTADO
        // ----------------------------------------------
        _ => None,
    }
}
