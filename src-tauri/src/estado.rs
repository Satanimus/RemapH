// ======================================================
// 🟢 Estado RemapH V3
// ------------------------------------------------------
// Estado global del perfil activo.
//
// Este módulo solo conoce si el perfil está activo.
// No conoce Runtime.
// No conoce Cache.
// No conoce Configuracion.
// ======================================================

use std::sync::{
    OnceLock,
    RwLock,
};


// ======================================================
// 🧠 ESTADO
// ======================================================

static ACTIVO:

    OnceLock<RwLock<bool>>

    = OnceLock::new();


// ======================================================
// 🔒 OBTENER ESTADO
// ======================================================

fn estado()

    -> &'static RwLock<bool>

{

    ACTIVO.get_or_init(

        || RwLock::new(true)

    )

}


// ======================================================
// 🟢 ACTIVAR
// ======================================================

pub fn activar() {

    *estado()
        .write()
        .unwrap()
        = true;

}


// ======================================================
// 🔴 DESACTIVAR
// ======================================================

pub fn desactivar() {

    *estado()
        .write()
        .unwrap()
        = false;

}


// ======================================================
// ❓ ESTADO
// ======================================================

pub fn esta_activo()

    -> bool

{

    *estado()
        .read()
        .unwrap()

}
