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


// ======================================================
// ⚙️ COMPILAR PERFIL
// ======================================================

pub fn compilar_perfil(

    perfil:
        &PerfilJson,

)

    -> Vec<RemapeoCache>

{

    perfil

        .remapeos

        .iter()

        .filter_map(

            compilar_remapeo

        )

        .collect()

}


// ======================================================
// ⚡ COMPILAR
// ======================================================

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