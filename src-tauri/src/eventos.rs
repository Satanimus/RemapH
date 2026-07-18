// ======================================================
// 📦 Eventos RemapH V3
// ------------------------------------------------------
// Representación genérica de entradas físicas.
//
// El Runtime no conoce:
//   - Teclado.
//   - Mouse.
//   - Joystick.
//
// Todo se representa mediante InputId.
// ======================================================

use serde::{
    Deserialize,
    Serialize,
};

use std::hash::Hash;


// ======================================================
// 🆔 IDENTIDAD DE INPUT
// ======================================================

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct InputId(
    String
);


// ======================================================
// 🏗️ CONSTRUCTOR
// ======================================================

impl InputId {

    pub fn new(

        fuente:
            &str,

        control:
            &str,

    ) -> Self {

        Self(

            format!(
                "{}:{}",
                fuente,
                control
            )

        )

    }

    // ==================================================
    // 🧩 FUENTE
    // ==================================================

    pub fn fuente(

        &self,

    ) -> Option<&str> {

        self.0
            .split_once(':')
            .map(
                |(fuente, _)| fuente
            )

    }


    // ==================================================
    // 🎛️ CONTROL
    // ==================================================

    pub fn control(

        &self,

    ) -> Option<&str> {

        self.0
            .split_once(':')
            .map(
                |(_, control)| control
            )

    }

}


// ======================================================
// 🔄 ESTADO
// ======================================================

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
pub enum InputState {

    Down,

    Up,

    Pulse,

}


// ======================================================
// 📡 EVENTO FÍSICO GENÉRICO
// ======================================================

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
pub struct InputEvent {

    pub input:
        InputId,

    pub state:
        InputState,

}


// ======================================================
// 🧱 CONSTRUCTORES
// ======================================================

impl InputEvent {

    pub fn down(

        input:
            InputId,

    ) -> Self {

        Self {

            input,

            state:
                InputState::Down,

        }

    }


    pub fn up(

        input:
            InputId,

    ) -> Self {

        Self {

            input,

            state:
                InputState::Up,

        }

    }


    pub fn pulse(

        input:
            InputId,

    ) -> Self {

        Self {

            input,

            state:
                InputState::Pulse,

        }

    }

}


// ======================================================
// 🔁 COMPATIBILIDAD INTERNA
// ======================================================

pub type Evento = InputEvent;