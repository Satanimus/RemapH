// ======================================================
// 🎛️ Pulsadores RemapH V3
// ------------------------------------------------------
// Diccionario interno de entradas.
//
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


            let mut pulsadores =
                Vec::new();


            for (numero, linea)
                in texto.lines().enumerate()
            {

                let linea =
                    linea.trim();


                if linea.is_empty()
                    || linea.starts_with('#')
                {

                    continue;

                }


                if numero == 0 {

                    continue;

                }


                let columnas:
                    Vec<&str>
                =
                    linea.split('\t').collect();


                if columnas.len() != 3 {

                    panic!(

                        "❌ Error interno en pulsadores.tsv. Línea {}",

                        numero + 1

                    );

                }


                let interno =
                    columnas[0].trim();


                let interception =
                    columnas[1].trim();


                let ui =
                    columnas[2].trim();


                if interno.is_empty() {

                    panic!(

                        "❌ Pulsador sin nombre interno. Línea {}",

                        numero + 1

                    );

                }


                if pulsadores.iter().any(

                    |pulsador: &Pulsador|

                        pulsador.interno == interno

                ) {

                    panic!(

                        "❌ Pulsador interno duplicado: {}",

                        interno

                    );

                }


                pulsadores.push(

                    Pulsador {

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