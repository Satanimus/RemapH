// ======================================================
// 🚀 Back_Salida RemapH V3
// ------------------------------------------------------
// Router de outputs físicos.
//
// El Runtime emite Accion genérica.
// Este módulo decide qué backend físico puede emitirla.
//
// El dispositivo de salida se descubre aquí.
// Nunca se reutiliza el device recibido por Entrada.
//
// Full:
//   - Interception
//
// Portable:
//   - SendInput
// ======================================================

use interception::{
    Interception,
    KeyState,
    Stroke,
};

use crate::perfilcache::AccionCache;


// ======================================================
// 📦 SALIDA
// ======================================================

pub struct Salida {

    ict:
        Interception,

    teclado:
        interception::Device,

    mouse:
        interception::Device,

}


// ======================================================
// 🚀 CREAR
// ======================================================

pub fn crear()

    -> Salida

{

    let ict =

        Interception::new()

        .expect(
            "No se pudo iniciar Interception para salida"
        );


    let teclado =

        encontrar_teclado(

            &ict

        )

        .expect(
            "No se encontró teclado de salida"
        );


    let mouse =

        encontrar_mouse(

            &ict

        )

        .expect(
            "No se encontró mouse de salida"
        );


    println!(
        "📤 Salida inicializada."
    );


    Salida {

        ict,

        teclado,

        mouse,

    }

}


// ======================================================
// 🎯 EJECUTAR ACCIÓN
// ======================================================

impl Salida {

    pub fn ejecutar(

        &self,

        accion:
            AccionCache,

    ) {

        match accion {

            AccionCache::Emitir(input) => {

                match input.fuente() {

                    // ----------------------------------
                    // 🎹 TECLADO
                    // ----------------------------------

                    Some("keyboard") => {

                        self.emitir_teclado(

                            &input

                        );

                    }


                    // ----------------------------------
                    // 🖱️ MOUSE
                    // ----------------------------------

                    Some("mouse") => {

                        self.emitir_mouse(

                            &input

                        );

                    }


                    // ----------------------------------
                    // ❌ NO SOPORTADO
                    // ----------------------------------

                    _ => {

                        println!(

                            "⚠️ Output no soportado: {:?}",

                            input

                        );

                    }

                }

            }

        }

    }


    // ==================================================
    // 🎹 EMITIR TECLADO
    // ==================================================

    fn emitir_teclado(

        &self,

        input:
            &crate::eventos::InputId,

    ) {

        let Some(code) =

            crate::backend::back_teclas::convertir_salida(

                input

            )

        else {

            println!(

                "⚠️ Tecla no soportada: {:?}",

                input

            );

            return;

        };


        let strokes = [

            Stroke::Keyboard {

                code,

                state:
                    KeyState::DOWN,

                information:
                    0,

            },


            Stroke::Keyboard {

                code,

                state:
                    KeyState::UP,

                information:
                    0,

            },

        ];


        println!(

            "📤 Emitiendo teclado: {:?}",

            input

        );


        let resultado =

            self.ict.send(

                self.teclado,

                &strokes,

            );


        println!(

            "📤 Resultado teclado: {}",

            resultado

        );

    }


    // ==================================================
    // 🖱️ EMITIR MOUSE
    // ==================================================

    fn emitir_mouse(

        &self,

        input:
            &crate::eventos::InputId,

    ) {

        let Some(output) =

            crate::backend::back_mouse::convertir_salida(

                input

            )

        else {

            println!(

                "⚠️ Mouse no soportado: {:?}",

                input

            );

            return;

        };


        match output {

            crate::backend::back_mouse::MouseOutput::Button {

                down,

                up,

            } => {

                let abajo =

                    Stroke::Mouse {

                        state:
                            down,

                        flags:
                            interception::MouseFlags::empty(),

                        rolling:
                            0,

                        x:
                            0,

                        y:
                            0,

                        information:
                            0,

                    };


                let arriba =

                    Stroke::Mouse {

                        state:
                            up,

                        flags:
                            interception::MouseFlags::empty(),

                        rolling:
                            0,

                        x:
                            0,

                        y:
                            0,

                        information:
                            0,

                    };


                self.ict.send(

                    self.mouse,

                    &[

                        abajo,

                        arriba,

                    ],

                );

            }


            crate::backend::back_mouse::MouseOutput::Wheel(

                rolling

            ) => {

                let stroke =

                    Stroke::Mouse {

                        state:
                            interception::MouseFilter::WHEEL,

                        flags:
                            interception::MouseFlags::empty(),

                        rolling,

                        x:
                            0,

                        y:
                            0,

                        information:
                            0,

                    };


                self.ict.send(

                    self.mouse,

                    &[stroke],

                );

            }

        }

    }

}


// ======================================================
// 🔎 BUSCAR TECLADO
// ======================================================

fn encontrar_teclado(

    ict:
        &Interception,

) -> Option<interception::Device> {

    let mut buffer =

        [0u8; 4096];


    for device in 1..=10 {

        if !interception::is_keyboard(

            device

        ) {

            continue;

        }


        if ict.get_hardware_id(

            device,

            &mut buffer,

        ) > 0 {

            return Some(

                device

            );

        }

    }


    None

}


// ======================================================
// 🔎 BUSCAR MOUSE
// ======================================================

fn encontrar_mouse(

    ict:
        &Interception,

) -> Option<interception::Device> {

    let mut buffer =

        [0u8; 4096];


    for device in 11..=20 {

        if !interception::is_mouse(

            device

        ) {

            continue;

        }


        if ict.get_hardware_id(

            device,

            &mut buffer,

        ) > 0 {

            return Some(

                device

            );

        }

    }


    None

}