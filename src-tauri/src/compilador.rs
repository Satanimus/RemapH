// ======================================================
// 🔨 Compilador RemapH V3
// ======================================================
//
// PerfilJson
//     ↓
// Compilador
//     ↓
// PerfilCache
//     ↓
// Cache
// ======================================================

use crate::cache;

use crate::eventos::InputId;

use crate::perfilcache::{
    AccionCache,
    AppCache,
    RemapeoCache,
    TriggerCache,
};

use crate::perfiljson::{
    AppJson,
    PerfilJson,
    RemapeoJson,
};

use std::collections::HashSet;

// ======================================================
// ⚙️ COMPILAR PERFIL
// ======================================================

pub fn compilar_perfil(

    perfil:
        &PerfilJson,

) -> Vec<RemapeoCache>

{

    let conflictivos =
        indices_conflictivos(

            perfil

        );


    perfil

        .remapeos

        .iter()

        .enumerate()

        .filter_map(

            |(indice, remapeo)| {

                if conflictivos.contains(

                    &indice

                ) {

                    return None;

                }


                compilar_remapeo(

                    remapeo

                )

            }

        )

        .collect()

}
// ======================================================
// ⚡ COMPILAR
// ======================================================

// ======================================================
// ⚠️ ÍNDICES CONFLICTIVOS
// ======================================================

fn indices_conflictivos(

    perfil:
        &PerfilJson

) -> HashSet<usize>

{

    let mut resultado =
        HashSet::new();


    for indice_a in 0..perfil.remapeos.len() {

        for indice_b in (indice_a + 1)..perfil.remapeos.len() {

            let fila_a =
                &perfil.remapeos[indice_a];

            let fila_b =
                &perfil.remapeos[indice_b];


            if !triggers_iguales(

                fila_a,

                fila_b

            ) {

                continue;

            }


            if !apps_conflictivas(

                fila_a,

                fila_b

            ) {

                continue;

            }


            resultado.insert(

                indice_a

            );


            resultado.insert(

                indice_b

            );

        }

    }


    resultado

}


// ======================================================
// 🎯 TRIGGER IDÉNTICO
// ======================================================

fn triggers_iguales(

    fila_a:
        &RemapeoJson,

    fila_b:
        &RemapeoJson

) -> bool

{

    let trigger_a =
        &fila_a.trigger;

    let trigger_b =
        &fila_b.trigger;


    let Some(gatillo_a) =
        &trigger_a.gatillo

    else {

        return false;

    };


    let Some(gatillo_b) =
        &trigger_b.gatillo

    else {

        return false;

    };


    if trigger_a.condicion != trigger_b.condicion {

        return false;

    }


    if trigger_a.modificadores.len()

        !=

        trigger_b.modificadores.len()

    {

        return false;

    }


    for indice in 0..trigger_a.modificadores.len() {

        let entrada_a =
            &trigger_a.modificadores[indice];

        let entrada_b =
            &trigger_b.modificadores[indice];


        if entrada_a != entrada_b {

            return false;

        }

    }


    gatillo_a == gatillo_b

}


// ======================================================
// 🖥️ APP INCOMPATIBLE
// ======================================================

fn apps_conflictivas(

    fila_a:
        &RemapeoJson,

    fila_b:
        &RemapeoJson

) -> bool

{

    match (

        &fila_a.app.programa,

        &fila_b.app.programa,

    ) {

        (

            None,

            None

        ) => true,


        (

            None,

            Some(_)

        ) => fila_b.app.segundo_plano,


        (

            Some(_),

            None

        ) => fila_a.app.segundo_plano,


        (

            Some(programa_a),

            Some(programa_b)

        ) =>

            programa_a.eq_ignore_ascii_case(

                programa_b

            ),

    }

}

pub fn compilar(

    perfil:
        &PerfilJson,

) {

    let remapeos =

        compilar_perfil(

            perfil

        );


    let cantidad =

        remapeos.len();


    cache::reemplazar(

        remapeos

    );


    println!(

        "🔨 {} remapeos compilados",

        cantidad

    );

}


// ======================================================
// 🧩 COMPILAR REMAPEO
// ======================================================

fn compilar_remapeo(

    remapeo:
        &RemapeoJson,

)

    -> Option<RemapeoCache>

{

    if remapeo.estado != "ON" {

        return None;

    }


    let gatillo =

        remapeo

            .trigger

            .gatillo

            .as_ref()?;


    let accion =

        remapeo

            .accion

            .as_ref()?;


    let accion_gatillo =

        accion

            .gatillo

            .as_ref()?;


    let modificadores =

        remapeo

            .trigger

            .modificadores

            .iter()

            .map(

                convertir_input

            )

            .collect();


    Some(

        RemapeoCache {

            app:

                convertir_app(

                    &remapeo.app

                ),

            trigger:

                TriggerCache {

                    modificadores,

                    gatillo:

                        convertir_input(

                            gatillo

                        ),

                },

            accion:

                AccionCache::Emitir(

                    convertir_input(

                        accion_gatillo

                    )

                ),

        }

    )

}


// ======================================================
// 🖥️ CONVERTIR APP
// ======================================================

fn convertir_app(

    app:
        &AppJson,

)

    -> AppCache

{

    match &app.programa {

        None =>

            AppCache::Global,


        Some(nombre) =>

            AppCache::Programa {

                nombre:
                    nombre.clone(),

                segundo_plano:
                    app.segundo_plano,

            },

    }

}


// ======================================================
// 🆔 INPUT → INPUT ID
// ======================================================

fn convertir_input(

    input:
        &crate::idioma::Input,

)

    -> InputId

{

    InputId::new(

        &input.fuente,

        &input.control,

    )

}