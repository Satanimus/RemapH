// ======================================================
// 🧠 Compilador
// RemapH V3
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
    RemapeoCache,
    TriggerCache,
};

use crate::perfiljson::{
    PerfilJson,
    RemapeoJson,
};

// ======================================================
// ⚙️ COMPILAR PERFIL
// ------------------------------------------------------
// Compila un perfil y devuelve su cache resultante.
// No modifica la cache global.
// ======================================================

pub fn compilar_perfil(
    perfil: &PerfilJson,
) -> Vec<RemapeoCache> {

    perfil
        .remapeos
        .iter()
        .filter_map(
            compilar_remapeo,
        )
        .collect()
}

// ======================================================
// ⚡ COMPILAR
// ------------------------------------------------------
// Compila y reemplaza la cache global.
// ======================================================

pub fn compilar(
    perfil: &PerfilJson,
) {

    let remapeos =
        compilar_perfil(
            perfil,
        );

    let cantidad =
        remapeos.len();

    cache::reemplazar(
        remapeos,
    );

    println!(
        " {} remapeos compilados",
        cantidad,
    );
}

// ======================================================
// 🧩 COMPILAR REMAPEO
// ======================================================

fn compilar_remapeo(
    remapeo: &RemapeoJson,
) -> Option<RemapeoCache> {

    if (
        remapeo.estado != "ON"
    ) {
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
                convertir_input,
            )
            .collect();

    Some(
        RemapeoCache {

            trigger:
                TriggerCache {

                    modificadores,

                    gatillo:
                        convertir_input(
                            gatillo,
                        ),
                },

            accion:
                AccionCache::Emitir(
                    convertir_input(
                        accion_gatillo,
                    ),
                ),
        },
    )
}

// ======================================================
// 🔄 INPUT → INPUT ID
// ======================================================

fn convertir_input(
    input: &crate::idioma::Input,
) -> InputId {

    InputId::new(
        &input.fuente,
        &input.control,
    )
}