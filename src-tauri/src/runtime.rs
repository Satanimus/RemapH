// ======================================================
// 🧠 Runtime RemapH V3
// ------------------------------------------------------
// Ejecuta remapeos compilados.
//
// El Runtime:
//   - No lee JSON.
//   - No interpreta configuraciones.
//   - No conoce Windows.
//   - No conoce Interception.
//
// Solo recibe InputEvent genérico.
//
// La entrada física pendiente pertenece a Entrada.
// El Runtime solo mantiene estado lógico.
//
// El orden de los inputs activos se conserva porque
// Cache utiliza el orden para resolver triggers.
// ======================================================

use std::collections::HashSet;
use std::sync::mpsc::Sender;

use crate::cache;
use crate::eventos::{
    InputEvent,
    InputId,
    InputState,
};
use crate::perfilcache::AccionCache;


// ======================================================
// ⚙️ RESULTADO
// ======================================================

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
)]
pub enum Resultado {

    Pasar,

    Esperar,

    Consumir,

}


// ======================================================
// 🧠 ESTADO
// ======================================================

pub struct Estado {

    orden_activos:
        Vec<InputId>,

    consumidos:
        HashSet<InputId>,

}


// ======================================================
// 🚀 CREAR
// ======================================================

impl Estado {

    pub fn nuevo()

        -> Self

    {

        Self {

            orden_activos:
                Vec::new(),

            consumidos:
                HashSet::new(),

        }

    }


    // ==================================================
    // 🎯 PROCESAR
    // ==================================================

    pub fn procesar(

        &mut self,

        evento:
            InputEvent,

        salida:
            &Sender<AccionCache>,

    ) -> Resultado {

        if !crate::estado::esta_activo() {

            return Resultado::Pasar;

        }


        match evento.state {

            InputState::Down => {

                self.procesar_down(

                    evento.input,

                    salida,

                )

            }


            InputState::Up => {

                self.procesar_up(

                    evento.input

                )

            }


            InputState::Pulse => {

                self.procesar_pulse(

                    evento.input,

                    salida,

                )

            }

        }

    }


    // ==================================================
    // ⬇️ DOWN
    // ==================================================

    fn procesar_down(

        &mut self,

        input:
            InputId,

        salida:
            &Sender<AccionCache>,

    ) -> Resultado {

        if self
            .consumidos
            .contains(&input)
        {

            return Resultado::Consumir;

        }


        if self
            .orden_activos
            .contains(&input)
        {

            return Resultado::Pasar;

        }


        self.orden_activos.push(

            input.clone()

        );


        if let Some(remapeo) =

            cache::buscar(

                &self.orden_activos,

                &input,

            )

        {

            for activo in

                &self.orden_activos

            {

                self.consumidos.insert(

                    activo.clone()

                );

            }


            salida

                .send(

                    remapeo.accion

                )

                .unwrap();


            return Resultado::Consumir;

        }


        if cache::tiene_prefijo(

            &self.orden_activos

        )

        {

            return Resultado::Esperar;

        }


        Resultado::Pasar

    }


    // ==================================================
    // ⬆️ UP
    // ==================================================

    fn procesar_up(

        &mut self,

        input:
            InputId,

    ) -> Resultado {

        self.orden_activos.retain(

            |activo|

                activo != &input

        );


        if self
            .consumidos
            .remove(&input)
        {

            return Resultado::Consumir;

        }


        Resultado::Pasar

    }


    // ==================================================
    // ⚡ PULSE
    // ==================================================

    fn procesar_pulse(

        &mut self,

        input:
            InputId,

        salida:
            &Sender<AccionCache>,

    ) -> Resultado {

        let Some(remapeo) =

            cache::buscar_pulse(

                &input

            )

        else {

            return Resultado::Pasar;

        };


        salida

            .send(

                remapeo.accion

            )

            .unwrap();


        Resultado::Consumir

    }

}


// ======================================================
// 🧪 TESTS
// ======================================================

#[cfg(test)]
mod tests {

    use super::*;

    use crate::cache;

    use crate::eventos::{
        InputEvent,
        InputId,
    };

    use crate::perfilcache::{
        AccionCache,
        RemapeoCache,
        TriggerCache,
    };


    fn teclado(

        nombre:
            &str,

    ) -> InputId {

        InputId::new(

            "keyboard",

            nombre,

        )

    }


    fn remapeo(

        modificadores:
            Vec<InputId>,

        gatillo:
            InputId,

        salida:
            InputId,

    ) -> RemapeoCache {

        RemapeoCache {

            trigger:

                TriggerCache {

                    modificadores,

                    gatillo,

                },

            accion:

                AccionCache::Emitir(

                    salida

                ),

        }

    }


    #[test]
    fn remapeo_simple_consumido() {

        let _lock =
            cache::bloquear_tests();


        cache::reemplazar(

            vec![

                remapeo(

                    vec![],

                    teclado("A"),

                    teclado("B"),

                )

            ]

        );


        let mut runtime =
            Estado::nuevo();


        let (tx, _rx) =
            std::sync::mpsc::channel();


        assert_eq!(

            runtime.procesar(

                InputEvent::down(

                    teclado("A")

                ),

                &tx,

            ),

            Resultado::Consumir

        );

    }


    #[test]
    fn remapeo_con_modificador_ciclo_completo() {

        let _lock =
            cache::bloquear_tests();


        cache::reemplazar(

            vec![

                remapeo(

                    vec![

                        teclado("LeftControl"),

                    ],

                    teclado("A"),

                    teclado("B"),

                )

            ]

        );


        let mut runtime =
            Estado::nuevo();


        let (tx, _rx) =
            std::sync::mpsc::channel();


        assert_eq!(

            runtime.procesar(

                InputEvent::down(

                    teclado("LeftControl")

                ),

                &tx,

            ),

            Resultado::Esperar

        );


        assert_eq!(

            runtime.procesar(

                InputEvent::down(

                    teclado("A")

                ),

                &tx,

            ),

            Resultado::Consumir

        );


        assert_eq!(

            runtime.procesar(

                InputEvent::up(

                    teclado("A")

                ),

                &tx,

            ),

            Resultado::Consumir

        );


        assert_eq!(

            runtime.procesar(

                InputEvent::up(

                    teclado("LeftControl")

                ),

                &tx,

            ),

            Resultado::Consumir

        );

    }


    #[test]
    fn pulse_remapeado() {

        let _lock =
            cache::bloquear_tests();


        cache::reemplazar(

            vec![

                remapeo(

                    vec![],

                    InputId::new(

                        "mouse",

                        "WheelUp",

                    ),

                    teclado("B"),

                )

            ]

        );


        let mut runtime =
            Estado::nuevo();


        let (tx, _rx) =
            std::sync::mpsc::channel();


        assert_eq!(

            runtime.procesar(

                InputEvent::pulse(

                    InputId::new(

                        "mouse",

                        "WheelUp",

                    )

                ),

                &tx,

            ),

            Resultado::Consumir

        );

    }

}