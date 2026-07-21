// ======================================================
// 🗃️ Cache RemapH V3
// ======================================================
//
// PerfilJson
//     ↓
// Compilador
//     ↓
// Cache compilada
//     ↓
// Cache activa
//     ↓
// Runtime
// ======================================================

use std::collections::HashSet;
use crate::perfilcache::CondicionTrigger;

use std::sync::{
    Mutex,
    OnceLock,
};

use crate::eventos::InputId;

use crate::perfilcache::{
    AppCache,
    RemapeoCache,
};


// ======================================================
// 📦 CACHE COMPILADA
// ======================================================

static CACHE:
    OnceLock<Mutex<Vec<RemapeoCache>>>
    = OnceLock::new();


// ======================================================
// ⚡ CACHE ACTIVA
// ======================================================

static CACHE_ACTIVA:
    OnceLock<Mutex<Vec<RemapeoCache>>>
    = OnceLock::new();


// ======================================================
// 🔒 LOCK TESTS
// ======================================================

#[cfg(test)]
static TEST_LOCK:
    OnceLock<Mutex<()>>
    = OnceLock::new();


// ======================================================
// 🔒 BLOQUEAR TESTS
// ======================================================

#[cfg(test)]
pub fn bloquear_tests()

    -> std::sync::MutexGuard<'static, ()>

{

    TEST_LOCK

        .get_or_init(

            ||

                Mutex::new(())

        )

        .lock()

        .unwrap()

}


// ======================================================
// 📦 OBTENER CACHE
// ======================================================

fn obtener_cache()

    -> &'static Mutex<Vec<RemapeoCache>>

{

    CACHE.get_or_init(

        ||

            Mutex::new(

                Vec::new()

            )

    )

}


// ======================================================
// ⚡ OBTENER CACHE ACTIVA
// ======================================================

fn obtener_cache_activa()

    -> &'static Mutex<Vec<RemapeoCache>>

{

    CACHE_ACTIVA.get_or_init(

        ||

            Mutex::new(

                Vec::new()

            )

    )

}


// ======================================================
// 🔄 REEMPLAZAR CACHE
// ======================================================

pub fn reemplazar(

    remapeos:
        Vec<RemapeoCache>,

) {

    {

        let mut cache =

            obtener_cache()

                .lock()

                .unwrap();


        *cache =

            remapeos.clone();

    }


    let mut cache_activa =

        obtener_cache_activa()

            .lock()

            .unwrap();


    *cache_activa =

        remapeos;

}


// ======================================================
// 🗑️ BORRAR
// ======================================================

pub fn borrar() {

    reemplazar(

        Vec::new()

    );

}


// ======================================================
// ❓ CACHE VACÍA
// ======================================================

pub fn esta_vacia()

    -> bool

{

    obtener_cache()

        .lock()

        .unwrap()

        .is_empty()

}


// ======================================================
// 🖥️ ACTUALIZAR CONTEXTO APP
// ======================================================

pub fn actualizar_contexto(

    programa_activo:
        Option<&str>,

    procesos_activos:
        &HashSet<String>,

) {

    let cache =

        obtener_cache()

            .lock()

            .unwrap();


    let remapeos =

        cache

            .iter()

            .filter(

                |remapeo|

                    app_activa(

                        &remapeo.app,

                        programa_activo,

                        procesos_activos,

                    )

            )

            .cloned()

            .collect();


    drop(cache);


    let mut cache_activa =

        obtener_cache_activa()

            .lock()

            .unwrap();


    *cache_activa =

        remapeos;

}


// ======================================================
// 🖥️ ¿APP ACTIVA?
// ======================================================

fn app_activa(

    app:
        &AppCache,

    programa_activo:
        Option<&str>,

    procesos_activos:
        &HashSet<String>,

)

    -> bool

{

    match app {

        AppCache::Global =>

            true,


        AppCache::Programa {

            nombre,

            segundo_plano,

        } => {

            if *segundo_plano {

                return procesos_activos.contains(

                    &nombre.to_lowercase()

                );

            }


            programa_activo

                .map(

                    |programa|

                        programa

                            .eq_ignore_ascii_case(

                                nombre

                            )

                )

                .unwrap_or(false)

        }

    }

}


// ======================================================
// 🎯 BUSCAR TRIGGER
// ======================================================

pub fn buscar(

    activos:
        &[InputId],

    gatillo:
        &InputId,

)

    -> Option<RemapeoCache>

{

    let cache =

        obtener_cache_activa()

            .lock()

            .unwrap();


    cache

        .iter()

        .find(

            |remapeo| {

                if remapeo.trigger.gatillo != *gatillo {

                    return false;

                }


                let modificadores =

                    &remapeo

                        .trigger

                        .modificadores;


                if activos.len()

                    != modificadores.len() + 1

                {

                    return false;

                }


                &activos[..modificadores.len()]

                    == modificadores.as_slice()

            }

        )

        .cloned()

}


// ======================================================
// ⚡ BUSCAR PULSE
// ======================================================

pub fn buscar_pulse(

    gatillo:
        &InputId,

)

    -> Option<RemapeoCache>

{

    let cache =

        obtener_cache_activa()

            .lock()

            .unwrap();


    cache

        .iter()

        .find(

            |remapeo| {

                remapeo

                    .trigger

                    .modificadores

                    .is_empty()

                    &&

                    remapeo

                        .trigger

                        .gatillo

                        == *gatillo

            }

        )

        .cloned()

}


// ======================================================
// ⏳ ¿TIENE PREFIJO?
// ======================================================

pub fn tiene_prefijo(

    activos:
        &[InputId],

)

    -> bool

{

    if activos.is_empty() {

        return false;

    }


    let cache =

        obtener_cache_activa()

            .lock()

            .unwrap();


    cache

        .iter()

        .any(

            |remapeo| {

                let modificadores =

                    &remapeo

                        .trigger

                        .modificadores;


                activos.len()

                    <= modificadores.len()

                    &&

                    modificadores

                        .starts_with(

                            activos

                        )

            }

        )

}


// ======================================================
// 🧪 TESTS
// ======================================================

#[cfg(test)]
mod tests {
    use crate::perfilcache::CondicionTrigger;

    use super::*;

    use crate::eventos::InputId;

    use crate::perfilcache::{
        AccionCache,
        AppCache,
        TriggerCache,
    };


    fn teclado(

        nombre:
            &str,

    )

        -> InputId

    {

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

    )

        -> RemapeoCache

    {

        RemapeoCache {

            app:

                AppCache::Global,

            trigger: TriggerCache {

                modificadores,

                gatillo,

                condicion:
                    CondicionTrigger::Simple,

            },

            accion:

                AccionCache::Emitir(

                    teclado("B")

                ),

        }

    }


    #[test]

    fn trigger_simple_coincide() {

        let _lock =

            bloquear_tests();


        reemplazar(

            vec![

                remapeo(

                    vec![],

                    teclado("A"),

                )

            ]

        );


        let activos =

            vec![

                teclado("A")

            ];


        assert!(

            buscar(

                &activos,

                &teclado("A"),

            )

            .is_some()

        );

    }


    #[test]

    fn trigger_con_modificador_coincide_en_orden() {

        let _lock =

            bloquear_tests();


        reemplazar(

            vec![

                remapeo(

                    vec![

                        teclado("LeftControl")

                    ],

                    teclado("A"),

                )

            ]

        );


        let activos =

            vec![

                teclado("LeftControl"),

                teclado("A"),

            ];


        assert!(

            buscar(

                &activos,

                &teclado("A"),

            )

            .is_some()

        );

    }


    #[test]

    fn trigger_con_modificador_no_coincide_en_orden_incorrecto() {

        let _lock =

            bloquear_tests();


        reemplazar(

            vec![

                remapeo(

                    vec![

                        teclado("LeftControl")

                    ],

                    teclado("A"),

                )

            ]

        );


        let activos =

            vec![

                teclado("A"),

                teclado("LeftControl"),

            ];


        assert!(

            buscar(

                &activos,

                &teclado("A"),

            )

            .is_none()

        );

    }


    #[test]

    fn trigger_incompleto_es_prefijo() {

        let _lock =

            bloquear_tests();


        reemplazar(

            vec![

                remapeo(

                    vec![

                        teclado("LeftControl")

                    ],

                    teclado("A"),

                )

            ]

        );


        let activos =

            vec![

                teclado("LeftControl")

            ];


        assert!(

            tiene_prefijo(

                &activos

            )

        );

    }


    #[test]

    fn input_ajeno_no_es_prefijo() {

        let _lock =

            bloquear_tests();


        reemplazar(

            vec![

                remapeo(

                    vec![

                        teclado("LeftControl")

                    ],

                    teclado("A"),

                )

            ]

        );


        let activos =

            vec![

                teclado("LeftShift")

            ];


        assert!(

            !tiene_prefijo(

                &activos

            )

        );

    }

}