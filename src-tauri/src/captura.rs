// ======================================================
// 🎹 Captura RemapH V3
// ------------------------------------------------------
// Captura inputs físicos genéricos.
//
// No conoce:
//   - Interception.
//   - Windows API.
//
// Recibe InputEvent genérico.
// ======================================================

use std::sync::{
    Mutex,
    OnceLock,
};

use crate::eventos::InputEvent;


// ======================================================
// 🧠 ESTADO
// ======================================================

struct EstadoCaptura {

    activa:
        bool,

    ultimo:
        Vec<String>,

}


static CAPTURA:

    OnceLock<Mutex<EstadoCaptura>>

    = OnceLock::new();


// ======================================================
// 🔒 ESTADO
// ======================================================

fn estado()

    -> &'static Mutex<EstadoCaptura>

{

    CAPTURA.get_or_init(

        || {

            Mutex::new(

                EstadoCaptura {

                    activa:
                        false,

                    ultimo:
                        Vec::new(),

                }

            )

        }

    )

}


// ======================================================
// 🚀 INICIAR
// ======================================================

pub fn iniciar() {

    let mut captura =

        estado()
            .lock()
            .unwrap();


    captura.activa =
        true;


    captura.ultimo.clear();

}


// ======================================================
// 🛑 FINALIZAR
// ======================================================

pub fn finalizar() {

    estado()
        .lock()
        .unwrap()
        .activa = false;

}


// ======================================================
// ❓ ACTIVA
// ======================================================

pub fn activa()

    -> bool

{

    estado()
        .lock()
        .unwrap()
        .activa

}


// ======================================================
// 📥 GUARDAR
// ======================================================

pub fn guardar(

    valor:
        String,

) {

    estado()
        .lock()
        .unwrap()
        .ultimo
        .push(valor);

}


// ======================================================
// 📤 OBTENER
// ======================================================

pub fn obtener()

    -> Vec<String>

{

    estado()
        .lock()
        .unwrap()
        .ultimo
        .clone()

}


// ======================================================
// 🎯 PROCESAR
// ======================================================

pub fn procesar(

    evento:
        &InputEvent,

) -> bool {

    if !activa() {

        return false;

    }


    let valor =

        evento
            .input
            .control()
            .unwrap_or("Unknown")
            .to_string();


    guardar(valor);


    finalizar();


    true

}