// ======================================================
// 🖱️ Back_Interception
// ------------------------------------------------------
// Backend físico de ENTRADA.
//
// Importante:
// Este módulo SOLO recibe y traduce inputs.
//
// No ejecuta acciones.
// No resuelve outputs.
// No conoce el Runtime.
// ======================================================

use interception::{Filter, Interception, KeyFilter, MouseFilter, Stroke};

// ======================================================
// 🚀 CREAR BACKEND
// ======================================================

pub fn crear() -> Interception {
    let ict = Interception::new().expect("No se pudo iniciar Interception");

    // ----------------------------------------------
    // 🎹 Teclado
    // ----------------------------------------------

    ict.set_filter(
        interception::is_keyboard,
        Filter::KeyFilter(KeyFilter::DOWN | KeyFilter::UP),
    );

    // ----------------------------------------------
    // 🖱️ Mouse
    // ----------------------------------------------

    ict.set_filter(
        interception::is_mouse,
        Filter::MouseFilter(
            MouseFilter::LEFT_BUTTON_DOWN
                | MouseFilter::LEFT_BUTTON_UP
                | MouseFilter::RIGHT_BUTTON_DOWN
                | MouseFilter::RIGHT_BUTTON_UP
                | MouseFilter::MIDDLE_BUTTON_DOWN
                | MouseFilter::MIDDLE_BUTTON_UP
                | MouseFilter::BUTTON_4_DOWN
                | MouseFilter::BUTTON_4_UP
                | MouseFilter::BUTTON_5_DOWN
                | MouseFilter::BUTTON_5_UP
                | MouseFilter::WHEEL,
        ),
    );

    println!("📥 Backend de entrada iniciado.");

    ict
}

// ======================================================
// 📥 RECIBIR
// ======================================================

pub fn recibir(ict: &Interception) -> Option<(interception::Device, Stroke)> {
    let device = ict.wait();

    let mut strokes = [Stroke::Mouse {
        state: MouseFilter::empty(),

        flags: interception::MouseFlags::empty(),

        rolling: 0,

        x: 0,

        y: 0,

        information: 0,
    }];

    if ict.receive(device, &mut strokes) <= 0 {
        return None;
    }

    Some((device, strokes[0]))
}

// ======================================================
// 🔁 REENVIAR ORIGINAL
// ======================================================

pub fn reenviar(ict: &Interception, device: interception::Device, stroke: Stroke) {
    ict.send(device, &[stroke]);
}

// ======================================================
// 🔄 TRADUCIR
// ======================================================

pub fn traducir(stroke: &Stroke) -> Option<crate::eventos::Evento> {
    match stroke {
        Stroke::Keyboard { code, state, .. } => Some(crate::backend::back_teclas::convertir(
            *code,
            *state == interception::KeyState::DOWN,
        )),

        Stroke::Mouse { state, rolling, .. } => {
            crate::backend::back_mouse::convertir(*state, *rolling)
        }
    }
}
