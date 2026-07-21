// ======================================================
// 🎛️ Pulsadores RemapH V3
// ------------------------------------------------------
// Diccionario interno de entradas.
//
// nativo
//     ↓
// interno
//     ↓
// interception
//     ↓
// ui
//
// El archivo TSV es interno al programa.
// No se modifica desde la interfaz.
// ======================================================

use std::sync::OnceLock;


// ======================================================
// 📦 MODELO PULSADOR
// ======================================================

#[derive(
    Clone,
    Debug,
)]
pub struct Pulsador {

    pub nativo:
        String,

    pub interno:
        String,

    pub interception:
        String,

    pub ui:
        String,

}


// ======================================================
// 🗂️ DICCIONARIO
// ======================================================

static PULSADORES:
    OnceLock<Vec<Pulsador>>
=
    OnceLock::new();


// ======================================================
// 📖 CARGAR DICCIONARIO
// ======================================================

fn cargar()
    -> &'static Vec<Pulsador>
{

    PULSADORES.get_or_init(

        || {

            let texto =
                include_str!(
                    "pulsadores.tsv"
                );


            let mut pulsadores:
                Vec<Pulsador>
            =
                Vec::new();


            for (numero_linea, linea)
                in texto.lines().enumerate()
            {

                let linea =
                    linea.trim();


                if linea.is_empty()
                    || linea.starts_with('#')
                {
                    continue;
                }


                if numero_linea == 0 {
                    continue;
                }


                let columnas:
                    Vec<&str>
                =
                    linea.split('\t').collect();


                if columnas.len() != 4 {

                    panic!(

                        "❌ Error interno en pulsadores.tsv. Línea {}",

                        numero_linea + 1

                    );

                }


                let nativo =
                    columnas[0].trim();


                let interno =
                    columnas[1].trim();


                let interception =
                    columnas[2].trim();


                let ui =
                    columnas[3].trim();



                if interno.is_empty() {

                    panic!(

                        "❌ Pulsador sin interno. Línea {}",

                        numero_linea + 1

                    );

                }


                if pulsadores.iter().any(

                    |p: &Pulsador|

                        p.interno == interno

                ) {

                    panic!(

                        "❌ Interno duplicado: {}",

                        interno

                    );

                }


                if !nativo.is_empty()
                    && pulsadores.iter().any(

                        |p: &Pulsador|

                            p.nativo == nativo

                    )
                {

                    panic!(

                        "❌ Nativo duplicado: {}",

                        nativo

                    );

                }



                pulsadores.push(

                    Pulsador {

                        nativo:
                            nativo.to_string(),

                        interno:
                            interno.to_string(),

                        interception:
                            interception.to_string(),

                        ui:
                            ui.to_string(),

                    }

                );

            }


            pulsadores

        }

    )

}


// ======================================================
// 🔍 BUSCAR POR NATIVO
// ======================================================

pub fn por_nativo(

    nativo:
        &str,

)
    -> Option<&'static Pulsador>
{

    cargar()

        .iter()

        .find(

            |pulsador|

                pulsador.nativo == nativo

        )

}


// ======================================================
// 🔍 BUSCAR POR INTERNO
// ======================================================

pub fn por_interno(

    interno:
        &str,

)
    -> Option<&'static Pulsador>
{

    cargar()

        .iter()

        .find(

            |pulsador|

                pulsador.interno == interno

        )

}


// ======================================================
// 🔍 BUSCAR POR INTERCEPTION
// ======================================================

pub fn por_interception(

    interception:
        &str,

)
    -> Option<&'static Pulsador>
{

    cargar()

        .iter()

        .find(

            |pulsador|

                pulsador.interception == interception

        )

}


// ======================================================
// 🔍 BUSCAR POR UI
// ======================================================

pub fn por_ui(

    ui:
        &str,

)
    -> Option<&'static Pulsador>
{

    cargar()

        .iter()

        .find(

            |pulsador|

                pulsador.ui == ui

        )

}


// ======================================================
// 📋 TODOS
// ======================================================

pub fn todos()

    -> &'static [Pulsador]
{

    cargar()

}


// ======================================================
// 🔄 CONVERSIONES
// ======================================================

pub fn nativo_a_interno(

    nativo:
        &str,

)
    -> Option<&'static str>
{

    por_nativo(nativo)

        .map(

            |p|

                p.interno.as_str()

        )

}


pub fn interno_a_nativo(

    interno:
        &str,

)
    -> Option<&'static str>
{

    por_interno(interno)

        .map(

            |p|

                p.nativo.as_str()

        )

}


pub fn interno_a_interception(

    interno:
        &str,

)
    -> Option<&'static str>
{

    por_interno(interno)

        .map(

            |p|

                p.interception.as_str()

        )

}


pub fn interno_a_ui(

    interno:
        &str,

)
    -> Option<&'static str>
{

    por_interno(interno)

        .map(

            |p|

                p.ui.as_str()

        )

}


pub fn interception_a_interno(

    interception:
        &str,

)
    -> Option<&'static str>
{

    por_interception(interception)

        .map(

            |p|

                p.interno.as_str()

        )

}


pub fn ui_a_interno(

    ui:
        &str,

)
    -> Option<&'static str>
{

    por_ui(ui)

        .map(

            |p|

                p.interno.as_str()

        )

}