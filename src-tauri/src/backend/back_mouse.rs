// ======================================================
// 🖱️ Back_Mouse RemapH V3
// ------------------------------------------------------
// Traduce mouse físico a InputEvent genérico.
// También describe outputs físicos de mouse.
// ======================================================

use interception::MouseFilter;

use crate::eventos::{
    Evento,
    InputId,
};


// ======================================================
// 🆔 CREAR INPUT DE MOUSE
// ======================================================

fn mouse(

    control:
        &str,

) -> InputId {

    InputId::new(

        "mouse",

        control,

    )

}


// ======================================================
// 📤 OUTPUT DE MOUSE
// ======================================================

pub enum MouseOutput {

    Button {

        down:
            MouseFilter,

        up:
            MouseFilter,

    },

    Wheel(

        i16

    ),

}


// ======================================================
// 📥 CONVERTIR ENTRADA
// ======================================================

pub fn convertir(

    state:
        MouseFilter,

    rolling:
        i16,

) -> Option<Evento> {

    // ----------------------------------------------
    // 🖱️ RUEDA
    // ----------------------------------------------

    if rolling > 0 {

        return Some(

            Evento::pulse(

                mouse(

                    "WheelUp"

                )

            )

        );

    }


    if rolling < 0 {

        return Some(

            Evento::pulse(

                mouse(

                    "WheelDown"

                )

            )

        );

    }


    // ----------------------------------------------
    // 🖱️ BOTONES
    // ----------------------------------------------

    if state.contains(

        MouseFilter::LEFT_BUTTON_DOWN

    ) {

        return Some(

            Evento::down(

                mouse(

                    "LeftButton"

                )

            )

        );

    }


    if state.contains(

        MouseFilter::LEFT_BUTTON_UP

    ) {

        return Some(

            Evento::up(

                mouse(

                    "LeftButton"

                )

            )

        );

    }


    if state.contains(

        MouseFilter::RIGHT_BUTTON_DOWN

    ) {

        return Some(

            Evento::down(

                mouse(

                    "RightButton"

                )

            )

        );

    }


    if state.contains(

        MouseFilter::RIGHT_BUTTON_UP

    ) {

        return Some(

            Evento::up(

                mouse(

                    "RightButton"

                )

            )

        );

    }


    if state.contains(

        MouseFilter::MIDDLE_BUTTON_DOWN

    ) {

        return Some(

            Evento::down(

                mouse(

                    "MiddleButton"

                )

            )

        );

    }


    if state.contains(

        MouseFilter::MIDDLE_BUTTON_UP

    ) {

        return Some(

            Evento::up(

                mouse(

                    "MiddleButton"

                )

            )

        );

    }


    if state.contains(

        MouseFilter::BUTTON_4_DOWN

    ) {

        return Some(

            Evento::down(

                mouse(

                    "Button4"

                )

            )

        );

    }


    if state.contains(

        MouseFilter::BUTTON_4_UP

    ) {

        return Some(

            Evento::up(

                mouse(

                    "Button4"

                )

            )

        );

    }


    if state.contains(

        MouseFilter::BUTTON_5_DOWN

    ) {

        return Some(

            Evento::down(

                mouse(

                    "Button5"

                )

            )

        );

    }


    if state.contains(

        MouseFilter::BUTTON_5_UP

    ) {

        return Some(

            Evento::up(

                mouse(

                    "Button5"

                )

            )

        );

    }


    // ----------------------------------------------
    // ❌ EVENTO NO SOPORTADO
    // ----------------------------------------------

    None

}


// ======================================================
// 📤 CONVERTIR OUTPUT
// ======================================================

pub fn convertir_salida(

    input:
        &InputId,

) -> Option<MouseOutput> {

    if input.fuente()
        != Some("mouse")
    {

        return None;

    }


    let control =

        input.control()?;


    match control {

        // ----------------------------------------------
        // 🖱️ BOTONES
        // ----------------------------------------------

        "LeftButton" => Some(

            MouseOutput::Button {

                down:
                    MouseFilter::LEFT_BUTTON_DOWN,

                up:
                    MouseFilter::LEFT_BUTTON_UP,

            }

        ),


        "RightButton" => Some(

            MouseOutput::Button {

                down:
                    MouseFilter::RIGHT_BUTTON_DOWN,

                up:
                    MouseFilter::RIGHT_BUTTON_UP,

            }

        ),


        "MiddleButton" => Some(

            MouseOutput::Button {

                down:
                    MouseFilter::MIDDLE_BUTTON_DOWN,

                up:
                    MouseFilter::MIDDLE_BUTTON_UP,

            }

        ),


        "Button4" => Some(

            MouseOutput::Button {

                down:
                    MouseFilter::BUTTON_4_DOWN,

                up:
                    MouseFilter::BUTTON_4_UP,

            }

        ),


        "Button5" => Some(

            MouseOutput::Button {

                down:
                    MouseFilter::BUTTON_5_DOWN,

                up:
                    MouseFilter::BUTTON_5_UP,

            }

        ),


        // ----------------------------------------------
        // 🖱️ RUEDA
        // ----------------------------------------------

        "WheelUp" => Some(

            MouseOutput::Wheel(

                120

            )

        ),


        "WheelDown" => Some(

            MouseOutput::Wheel(

                -120

            )

        ),


        // ----------------------------------------------
        // ❌ NO SOPORTADO
        // ----------------------------------------------

        _ => None,

    }

}