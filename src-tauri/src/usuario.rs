// ======================================================
// 👤 Usuario RemapH V3
// ------------------------------------------------------
// Dueño de las rutas y archivos del usuario.
//
// Usuario/
//   ├── perfil_Default.json
//   ├── perfil_Juegos.json
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
    Path,
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

fn rutas_perfiles()

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
// 📋 LISTAR PERFILES
// ======================================================

pub fn perfiles()

    -> Result<Vec<String>, String>

{

    let mut nombres =

        rutas_perfiles()?

            .into_iter()

            .filter_map(

                |ruta|

                    nombre_desde_ruta(

                        &ruta

                    )

            )

            .collect::<Vec<_>>();


    nombres.sort();


    Ok(

        nombres

    )

}


// ======================================================
// 🔎 ES PERFIL
// ======================================================

fn es_perfil(

    ruta:
        &Path,

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
// 🆔 NOMBRE DESDE RUTA
// ======================================================

fn nombre_desde_ruta(

    ruta:
        &Path,

)

    -> Option<String>

{

    let nombre =

        ruta

            .file_name()?

            .to_str()?;


    let nombre =

        nombre

            .strip_prefix(

                "perfil_"

            )?

            .strip_suffix(

                ".json"

            )?;


    Some(

        nombre.to_string()

    )

}


// ======================================================
// 📍 RUTA POR NOMBRE
// ======================================================

pub fn ruta_perfil(

    nombre:
        &str,

)

    -> Result<PathBuf, String>

{

    if nombre.trim().is_empty() {

        return Err(

            "El nombre del perfil está vacío"

                .to_string()

        );

    }


    if nombre.contains('/')

        ||

        nombre.contains('\\')

        ||

        nombre == "."

        ||

        nombre == ".."

    {

        return Err(

            "Nombre de perfil inválido"

                .to_string()

        );

    }


    Ok(

        carpeta()?

            .join(

                format!(

                    "perfil_{}.json",

                    nombre

                )

            )

    )

}


// ======================================================
// 🆕 PERFIL DEFAULT
// ======================================================

pub fn perfil_default()

    -> Result<PathBuf, String>

{

    ruta_perfil(

        "Default"

    )

}


// ======================================================
// 🕒 PERFIL ACTUAL
// ------------------------------------------------------
// El perfil actual es siempre el JSON modificado
// más recientemente.
//
// Si no existe ningún perfil, se usa Default.
// ======================================================

pub fn perfil_actual()

    -> Result<PathBuf, String>

{

    let perfiles =
        rutas_perfiles()?;


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

        return perfil_default();

    };


    Ok(

        perfil

    )

}


// ======================================================
// 🆔 NOMBRE PERFIL ACTUAL
// ======================================================

pub fn nombre_actual()

    -> Result<String, String>

{

    let ruta =

        perfil_actual()?;


    nombre_desde_ruta(

        &ruta

    )

    .ok_or_else(

        || {

            "No se pudo determinar el nombre del perfil"

                .to_string()

        }

    )

}