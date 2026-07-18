// ======================================================
// 🎮 Comandos Tauri RemapH V3
// ------------------------------------------------------
// Comandos expuestos a la interfaz.
//
// UI
//   ↓
// PerfilJson
//   ↓
// Usuario
//   ↓
// Persistencia
//   ↓
// Compilador
//   ↓
// Cache
// ======================================================

use crate::cache;
use crate::compilador;
use crate::estado;
use crate::persistencia;
use crate::perfiljson::{
    PerfilJson,
    RemapeoJson,
    TriggerJson,
};
use crate::usuario;

use serde::Deserialize;


// ======================================================
// 🧩 MODELO UI
// ======================================================

#[derive(
    Deserialize,
)]
pub struct FilaUI {

    pub id:
        String,

    pub estado:
        String,

    pub app:
        String,

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
// 🟢 ACTIVAR PERFIL
// ======================================================

#[tauri::command]
pub fn activar_perfil() {

    sincronizar_estado_cache();


    println!(

        "🟢 Perfil activado"

    );

}


// ======================================================
// 🔴 DESACTIVAR PERFIL
// ======================================================

#[tauri::command]
pub fn desactivar_perfil() {

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

) -> Result<(), String> {

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


    sincronizar_estado_cache();


    println!(

        "📦 Perfil guardado y compilado"

    );


    Ok(())

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


        sincronizar_estado_cache();


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


    sincronizar_estado_cache();


    Ok(

        perfil

    )

}


// ======================================================
// 🧠 SINCRONIZAR ESTADO CON CACHE
// ======================================================

fn sincronizar_estado_cache() {

    if cache::esta_vacia() {

        estado::desactivar();

    }

    else {

        estado::activar();

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

) -> RemapeoJson {

    RemapeoJson {

        id:
            fila.id,

        estado:
            fila.estado,

        app:
            fila.app,

        trigger:

            convertir_trigger(

                fila.trigger

            ),

        tipo:
            fila.tipo,

        accion:

            fila.accion.map(

                convertir_trigger

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
            trigger.condicion,

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