// ======================================================
// 👤 Usuario RemapH V3
// ------------------------------------------------------
// Dueño de las rutas y archivos del usuario.
//
// Usuario/
//   ├── perfil_default.json
//   ├── perfil_juegos.json
//   └── ...
//
// Este módulo:
//   - Resuelve la carpeta Usuario.
//   - Busca perfiles.
//   - Decide el perfil actual.
//
// No conoce Runtime.
// No conoce Cache.
// No compila perfiles.
// ======================================================

use std::fs;

use std::path::{
    PathBuf,
};

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};


// ======================================================
// 📁 OBTENER CARPETA USUARIO
// ======================================================

pub fn carpeta()

    -> Result<PathBuf, String>

{

    let appdata =

        std::env::var(

            "APPDATA"

        )

        .map_err(

            |error|

                error.to_string()

        )?;


    let carpeta =

        PathBuf::from(

            appdata

        )

        .join(

            "RemapH V3"

        )

        .join(

            "Usuario"

        );


    fs::create_dir_all(

        &carpeta

    )

    .map_err(

        |error|

            error.to_string()

    )?;


    Ok(

        carpeta

    )

}


// ======================================================
// 📄 BUSCAR PERFILES
// ======================================================

fn perfiles()

    -> Result<Vec<PathBuf>, String>

{

    let carpeta =
        carpeta()?;


    let mut perfiles =
        Vec::new();


    let entradas =

        fs::read_dir(

            &carpeta

        )

        .map_err(

            |error|

                error.to_string()

        )?;


    for entrada in entradas {

        let ruta =

            entrada

                .map_err(

                    |error|

                        error.to_string()

                )?

                .path();


        if !es_perfil(

            &ruta

        ) {

            continue;

        }


        perfiles.push(

            ruta

        );

    }


    Ok(

        perfiles

    )

}


// ======================================================
// 🔎 ES PERFIL
// ======================================================

fn es_perfil(

    ruta:
        &PathBuf,

) -> bool {

    let Some(nombre) =

        ruta

            .file_name()

            .and_then(

                |nombre|

                    nombre.to_str()

            )

    else {

        return false;

    };


    nombre.starts_with(

        "perfil_"

    )

    &&

    nombre.ends_with(

        ".json"

    )

}


// ======================================================
// 🆕 PERFIL DEFAULT
// ======================================================

pub fn perfil_default()

    -> Result<PathBuf, String>

{

    Ok(

        carpeta()?

            .join(

                "perfil_default.json"

            )

    )

}


// ======================================================
// 🕒 PERFIL MÁS RECIENTE
// ======================================================

pub fn perfil_actual()

    -> Result<PathBuf, String>

{

    let perfiles =
        perfiles()?;


    let Some(perfil) =

        perfiles

            .into_iter()

            .max_by_key(

                |ruta| {

                    fs::metadata(

                        ruta

                    )

                    .and_then(

                        |metadata|

                            metadata.modified()

                    )

                    .unwrap_or(

                        SystemTime::UNIX_EPOCH

                    )

                }

            )

    else {

        return Ok(

            perfil_default()?

        );

    };


    Ok(

        perfil

    )

}