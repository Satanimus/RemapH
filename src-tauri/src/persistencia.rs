// ======================================================
// 💾 Persistencia RemapH V3
// ------------------------------------------------------
// Guarda y carga PerfilJson.
//
// Este módulo:
//   - Escribe JSON.
//   - Lee JSON.
//
// No decide rutas.
// No busca perfiles.
// No compila.
// No toca Cache.
// No toca Runtime.
// ======================================================

use crate::perfiljson::PerfilJson;

use std::fs;

use std::path::Path;


// ======================================================
// 💾 GUARDAR
// ======================================================

pub fn guardar(

    perfil:
        &PerfilJson,

    ruta:
        &Path,

) -> Result<(), String> {

    let json =

        serde_json::to_string_pretty(

            perfil

        )

        .map_err(

            |error|

                error.to_string()

        )?;


    fs::write(

        ruta,

        json

    )

    .map_err(

        |error|

            error.to_string()

    )?;


    Ok(())

}


// ======================================================
// 📂 CARGAR
// ======================================================

pub fn cargar(

    ruta:
        &Path,

) -> Result<PerfilJson, String> {

    let json =

        fs::read_to_string(

            ruta

        )

        .map_err(

            |error|

                error.to_string()

        )?;


    serde_json::from_str(

        &json

    )

    .map_err(

        |error|

            error.to_string()

    )

}