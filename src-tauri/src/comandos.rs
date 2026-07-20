// ======================================================
// 🎮 Comandos Tauri RemapH V3
// ------------------------------------------------------
// Comandos expuestos a la interfaz.
//
// UI
//  ↓
// PerfilJson
//  ↓
// Usuario
//  ↓
// Persistencia
//  ↓
// Compilador
//  ↓
// Cache
// ======================================================

use std::fs;

use crate::cache;
use crate::compilador;
use crate::estado;
use crate::persistencia;
use crate::perfiljson::{
    AppJson,
    PerfilJson,
    RemapeoJson,
    TriggerJson,
};
use crate::usuario;

use serde::{
    Deserialize,
    Serialize,
};

// ======================================================
// 🧩 MODELO UI
// ======================================================

// ======================================================
// APP UI
// ======================================================
// Representa la configuración de la columna App
// recibida desde TypeScript.
// ======================================================

#[derive(
    Deserialize,
)]
pub struct AppUI {

    pub programa:
        Option<String>,

    #[serde(rename = "segundoPlano")]
    pub segundo_plano:
        bool,

}

#[derive(
    Deserialize,
)]
pub struct FilaUI {

    pub id:
        String,

    pub estado:
        String,

    pub app:
        AppUI,

    pub trigger:
        TriggerUI,

    pub tipo:
        String,

    pub accion:
        Option<TriggerUI>,

    pub condicion:
        String,

    pub ejecucion:
        String,

    pub color:
        String,

    pub nota:
        String,

}


// ======================================================
// 🎯 TRIGGER UI
// ======================================================

#[derive(
    Deserialize,
)]
pub struct TriggerUI {

    pub modificadores:
        Vec<EntradaUI>,

    pub gatillo:
        Option<EntradaUI>,

    pub condicion:
        String,

}


// ======================================================
// 🆔 ENTRADA UI
// ======================================================

#[derive(
    Deserialize,
)]
pub struct EntradaUI {

    pub tipo:
        String,

    pub codigo:
        String,

}

// ======================================================
// 📦 RESULTADO PERFIL
// ======================================================

#[derive(
    Serialize,
)]
pub struct ResultadoPerfil {

    pub perfil:
        PerfilJson,

    pub nombre:
        String,

    pub perfiles:
        Vec<String>,

    pub cache_activo:
        bool,

}

// ======================================================
// 🟢🔴 ESTADO CACHE DE PERFIL
// ======================================================

#[derive(
    Serialize,
)]
pub struct EstadoCachePerfil {

    pub nombre: String,

    pub cache_activo: bool,
}

// ======================================================
// 🟢 ACTIVAR PERFIL
// ======================================================

#[tauri::command]
pub fn activar_perfil() -> Result<bool, String> {

    let ruta =

        usuario::perfil_actual()?;


    let perfil =

        persistencia::cargar(

            &ruta

        )?;


    compilador::compilar(

        &perfil

    );


    sincronizar_estado_cache(

        &perfil

    );


    println!(

        "🟢 Perfil activado"

    );


    Ok(

        !cache::esta_vacia()

    )

}


// ======================================================
// 🔴 DESACTIVAR PERFIL
// ======================================================

#[tauri::command]
pub fn desactivar_perfil() {

    cache::borrar();

    estado::desactivar();


    println!(

        "🔴 Perfil desactivado"

    );

}


// ======================================================
// 🔨 GUARDAR Y COMPILAR PERFIL
// ======================================================

#[tauri::command]
pub fn compilar_perfil(

    filas:
        Vec<FilaUI>,

) -> Result<bool, String> {

    let perfil =

        convertir_perfil(

            filas

        );


    let ruta =

        usuario::perfil_actual()?;


    persistencia::guardar(

        &perfil,

        &ruta,

    )?;


    compilador::compilar(

        &perfil

    );


    sincronizar_estado_cache(

        &perfil

    );


    let cache_activo =

        !cache::esta_vacia();


    println!(

        "📦 Perfil guardado y compilado"

    );


    Ok(

        cache_activo

    )

}


// ======================================================
// 📂 CARGAR PERFIL ACTUAL
// ======================================================

#[tauri::command]
pub fn obtener_perfil_actual()

    -> Result<PerfilJson, String>

{

    let ruta =

        usuario::perfil_actual()?;


    if !ruta.exists() {

        let perfil =

            PerfilJson::nuevo();


        persistencia::guardar(

            &perfil,

            &ruta,

        )?;


        compilador::compilar(

            &perfil

        );


        sincronizar_estado_cache(

            &perfil

        );


        return Ok(

            perfil

        );

    }


    let perfil =

        persistencia::cargar(

            &ruta

        )?;


    compilador::compilar(

        &perfil

    );


    sincronizar_estado_cache(

        &perfil

    );


    Ok(

        perfil

    )

}


// ======================================================
// 📋 OBTENER PERFILES
// ======================================================

#[tauri::command]
pub fn obtener_perfiles()

    -> Result<Vec<String>, String>

{

    usuario::perfiles()

}

// ======================================================
// 🟢🔴 OBTENER ESTADO CACHE DE PERFILES
// ======================================================

#[tauri::command]
pub fn obtener_estados_cache_perfiles(
) -> Result<
    Vec<EstadoCachePerfil>,
    String,
> {

    let nombres =
        usuario::perfiles()?;

    let mut resultado =
        Vec::new();

    for nombre in nombres {

        let ruta =
            usuario::ruta_perfil(
                &nombre,
            )?;

        let perfil =
            persistencia::cargar(
                &ruta,
            )?;

        let cache =
            compilador::compilar_perfil(
                &perfil,
            );

        resultado.push(
            EstadoCachePerfil {

                nombre,

                cache_activo:
                    !cache.is_empty(),
            },
        );
    }

    Ok(
        resultado,
    )
}

// ======================================================
// 🆔 OBTENER NOMBRE ACTUAL
// ======================================================

#[tauri::command]
pub fn obtener_nombre_perfil_actual()

    -> Result<String, String>

{

    usuario::nombre_actual()

}


// ======================================================
// 🟢 OBTENER ESTADO CACHE
// ======================================================

#[tauri::command]
pub fn obtener_estado_cache()

    -> bool

{

    !cache::esta_vacia()

}


// ======================================================
// 🔄 RESTAURAR PERFIL ACTUAL
// ======================================================

#[tauri::command]
pub fn restaurar_perfil_actual()

    -> Result<ResultadoPerfil, String>

{

    let ruta =

        usuario::perfil_actual()?;


    if !ruta.exists() {

        let perfil =

            PerfilJson::nuevo();


        persistencia::guardar(

            &perfil,

            &ruta,

        )?;

    }


    let perfil =

        persistencia::cargar(

            &ruta

        )?;


    let nombre =

        usuario::nombre_actual()?;


    resultado_perfil(

        perfil,

        nombre

    )

}


// ======================================================
// 📋 CLONAR PERFIL ACTUAL
// ======================================================

#[tauri::command]
pub fn clonar_perfil(

    filas:
        Vec<FilaUI>,

)

    -> Result<ResultadoPerfil, String>

{

    let nombre_actual =

        usuario::nombre_actual()?;


    let nombre =

        siguiente_nombre(

            &nombre_actual

        )?;


    let perfil =

        convertir_perfil(

            filas

        );


    cache::borrar();

    estado::desactivar();


    let ruta =

        usuario::ruta_perfil(

            &nombre

        )?;


    persistencia::guardar(

        &perfil,

        &ruta,

    )?;

    compilador::compilar(

        &perfil

    );

    sincronizar_estado_cache(

        &perfil

    );

    resultado_perfil(

        perfil,

        nombre

    )

}


// ======================================================
// ✏️ RENOMBRAR PERFIL ACTUAL
// ======================================================

#[tauri::command]
pub fn renombrar_perfil(

    nuevo_nombre:
        String,

)

    -> Result<ResultadoPerfil, String>

{

    let nombre_actual =

        usuario::nombre_actual()?;


    let nuevo_nombre =

        nuevo_nombre.trim();


    if nuevo_nombre.is_empty() {

        return Err(

            "El nombre del perfil está vacío"

                .to_string()

        );

    }


    if nuevo_nombre == nombre_actual {

        return Err(

            "El perfil ya tiene ese nombre"

                .to_string()

        );

    }


    let nuevo_nombre =

        siguiente_nombre(

            nuevo_nombre

        )?;


    let ruta_actual =

        usuario::perfil_actual()?;


    let nueva_ruta =

        usuario::ruta_perfil(

            &nuevo_nombre

        )?;


    cache::borrar();

    estado::desactivar();


    fs::rename(

        &ruta_actual,

        &nueva_ruta,

    )

    .map_err(

        |error|

            error.to_string()

    )?;


    let perfil =

        persistencia::cargar(

            &nueva_ruta

        )?;

        compilador::compilar(

            &perfil

        );

        sincronizar_estado_cache(

            &perfil

        );

    resultado_perfil(

        perfil,

        nuevo_nombre

    )

}


// ======================================================
// 🗑️ ELIMINAR PERFIL ACTUAL
// ======================================================

#[tauri::command]
pub fn eliminar_perfil_actual()

    -> Result<ResultadoPerfil, String>

{

    let ruta_actual =

        usuario::perfil_actual()?;


    cache::borrar();

    estado::desactivar();


    if ruta_actual.exists() {

        fs::remove_file(

            ruta_actual

        )

        .map_err(

            |error|

                error.to_string()

        )?;

    }


    let perfiles =

        usuario::perfiles()?;


    if let Some(nombre) =

        perfiles.first()

    {

        let ruta =

            usuario::ruta_perfil(

                nombre

            )?;


        let perfil =

            persistencia::cargar(

                &ruta

            )?;

            compilador::compilar(

                &perfil

            );

            sincronizar_estado_cache(

                &perfil

            );

        return resultado_perfil(

            perfil,

            nombre.to_string()

        );

    }


    let nombre =

        "Default".to_string();


    let perfil =

        PerfilJson::nuevo();


    let ruta =

        usuario::ruta_perfil(

            &nombre

        )?;


    persistencia::guardar(

        &perfil,

        &ruta,

    )?;


    resultado_perfil(

        perfil,

        nombre

    )

}


// ======================================================
// 🆕 CREAR PERFIL NUEVO
// ======================================================

#[tauri::command]
pub fn crear_perfil_nuevo()

    -> Result<ResultadoPerfil, String>

{

    cache::borrar();

    estado::desactivar();


    let nombre =

        siguiente_nombre(

            "Default"

        )?;


    let perfil =

        PerfilJson::nuevo();


    let ruta =

        usuario::ruta_perfil(

            &nombre

        )?;


    persistencia::guardar(

        &perfil,

        &ruta,

    )?;


    resultado_perfil(

        perfil,

        nombre

    )

}


// ======================================================
// 🔄 SELECCIONAR PERFIL
// ======================================================

#[tauri::command]
pub fn seleccionar_perfil(

    nombre:
        String,

)

    -> Result<ResultadoPerfil, String>

{

    let ruta =

        usuario::ruta_perfil(

            &nombre

        )?;


    if !ruta.exists() {

        return Err(

            "El perfil seleccionado no existe"

                .to_string()

        );

    }


    cache::borrar();

    estado::desactivar();


    let perfil =

        persistencia::cargar(

            &ruta

        )?;


    persistencia::guardar(

        &perfil,

        &ruta,

    )?;


    compilador::compilar(

        &perfil

    );


    sincronizar_estado_cache(

        &perfil

    );


    println!(

        "📂 Perfil seleccionado: {}",

        nombre

    );


    resultado_perfil(

        perfil,

        nombre

    )

}


// ======================================================
// 📦 CREAR RESULTADO
// ======================================================

fn resultado_perfil(

    perfil:
        PerfilJson,

    nombre:
        String,

)

    -> Result<ResultadoPerfil, String>

{

    Ok(

        ResultadoPerfil {

            perfil,

            nombre,

            perfiles:
                usuario::perfiles()?,

            cache_activo:
                !cache::esta_vacia(),

        }

    )

}


// ======================================================
// 🔢 SIGUIENTE NOMBRE DISPONIBLE
// ======================================================

fn siguiente_nombre(

    base:
        &str,

)

    -> Result<String, String>

{

    let ruta =

        usuario::ruta_perfil(

            base

        )?;


    if !ruta.exists() {

        return Ok(

            base.to_string()

        );

    }


    let mut numero =
        2;


    loop {

        let nombre =

            format!(

                "{} ({})",

                base,

                numero

            );


        let ruta =

            usuario::ruta_perfil(

                &nombre

            )?;


        if !ruta.exists() {

            return Ok(

                nombre

            );

        }


        numero += 1;

    }

}


// ======================================================
// 🔄 SINCRONIZAR ESTADO CON CACHE
// ======================================================

fn sincronizar_estado_cache(

    perfil:
        &PerfilJson

) {

    if perfil.remapeos.iter().any(

        |remapeo|

            remapeo.estado == "ON"

    ) {

        estado::activar();

    }

    else {

        estado::desactivar();

    }

}


// ======================================================
// 🔄 CONVERTIR PERFIL
// ======================================================

fn convertir_perfil(

    filas:
        Vec<FilaUI>,

) -> PerfilJson {

    let remapeos =

        filas

            .into_iter()

            .map(

                convertir_fila

            )

            .collect();


    PerfilJson {

        remapeos,

    }

}


// ======================================================
// 🧩 CONVERTIR FILA
// ======================================================

fn convertir_fila(
    fila:
    FilaUI,
)
-> RemapeoJson
{
    RemapeoJson {

        id:
        fila.id,

        estado:
        fila.estado,

        app:
        convertir_app(
            fila.app,
        ),

        trigger:
        convertir_trigger(
            fila.trigger,
        ),

        tipo:
        fila.tipo,

        accion:
        fila.accion
        .map(
            convertir_trigger,
        ),

        condicion:
        fila.condicion,

        ejecucion:
        fila.ejecucion,

        color:
        fila.color,

        nota:
        fila.nota,

    }
}

// ======================================================
// CONVERTIR APP
// ======================================================

fn convertir_app(
    app:
        AppUI,
)
-> AppJson
{
    AppJson {

        programa:
            app.programa,

        segundo_plano:
            app.segundo_plano,

    }
}

// ======================================================
// 🎯 CONVERTIR TRIGGER
// ======================================================

fn convertir_trigger(

    trigger:
        TriggerUI,

) -> TriggerJson {

    TriggerJson {

        modificadores:

            trigger

                .modificadores

                .into_iter()

                .map(

                    convertir_entrada

                )

                .collect(),

        gatillo:

            trigger

                .gatillo

                .map(

                    convertir_entrada

                ),

        condicion:

            trigger

                .condicion,

    }

}


// ======================================================
// 🆔 CONVERTIR ENTRADA
// ======================================================

fn convertir_entrada(

    entrada:
        EntradaUI,

) -> crate::idioma::Input {

    crate::idioma::Input::nuevo(

        convertir_fuente(

            &entrada.tipo

        ),

        &entrada.codigo,

    )

}


// ======================================================
// 🌐 TIPO UI → FUENTE INTERNA
// ======================================================

fn convertir_fuente(

    tipo:
        &str,

) -> &'static str {

    match tipo {

        "Teclado" =>
            "keyboard",

        "Mouse" =>
            "mouse",

        "Multimedia" =>
            "multimedia",

        "Joystick" =>
            "joystick",

        _ =>
            "unknown",

    }

}


// ======================================================
// 🎹 INICIAR CAPTURA
// ======================================================

#[tauri::command]
pub fn iniciar_captura() {

    crate::captura::iniciar();


    println!(

        "🎹 Captura iniciada"

    );

}


// ======================================================
// 📥 OBTENER CAPTURA
// ======================================================

#[tauri::command]
pub fn obtener_captura()

    -> Vec<String>

{

    crate::captura::obtener()

}

// ======================================================
// 🖥️ PROCESOS Y ÍCONOS (columna App)
// ------------------------------------------------------
// Comandos delgados: la lógica real vive en
// backend::back_procesos (capa Platform).
// ======================================================

use crate::backend::back_procesos;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;


#[derive(
    Serialize,
)]
pub struct IconoJson {

    pub ancho:
        u32,

    pub alto:
        u32,

    pub pixeles:
        String,

}


#[derive(
    Serialize,
)]
pub struct ProcesoIconoJson {

    pub nombre:
        String,

    pub icono:
        Option<IconoJson>,

}


fn convertir_icono(
    icono:
        back_procesos::IconoRaw,
) -> IconoJson {

    IconoJson {

        ancho:
            icono.ancho,

        alto:
            icono.alto,

        pixeles:
            BASE64.encode(icono.pixeles),

    }

}


// ======================================================
// 📋 LISTAR PROCESOS CON VENTANA VISIBLE
// ======================================================

#[tauri::command]
pub fn listar_procesos_ventana()

    -> Vec<ProcesoIconoJson>

{

    back_procesos::enumerar_procesos_ventana()

        .into_iter()

        .map(
            |proceso| {

                let icono =
                    back_procesos::extraer_icono(&proceso.ruta)
                        .map(convertir_icono);

                ProcesoIconoJson {

                    nombre:
                        proceso.nombre,

                    icono,

                }

            }
        )

        .collect()

}


// ======================================================
// 🎨 OBTENER ÍCONO DE UN PROGRAMA PUNTUAL
// ------------------------------------------------------
// Busca el proceso en ejecución cuyo nombre coincide y
// devuelve su ícono. Si no está corriendo, devuelve None
// (la UI usa el ícono genérico como respaldo).
// ======================================================

#[tauri::command]
pub fn obtener_icono_programa(
    nombre:
        String,
) -> Option<IconoJson> {

    let proceso =
        back_procesos::enumerar_procesos_ventana()
            .into_iter()
            .find(
                |proceso|
                    proceso.nombre.eq_ignore_ascii_case(&nombre)
            )?;

    back_procesos::extraer_icono(&proceso.ruta)
        .map(convertir_icono)

}