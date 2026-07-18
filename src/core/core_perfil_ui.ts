// ======================================================
// 📄 core_Perfil_UI RemapH V3
// ------------------------------------------------------
// Estado temporal editable de la interfaz.
//
// Este módulo:
//   - Mantiene el Perfil actual de la UI.
//   - Permite reemplazarlo al cargar JSON.
//
// PerfilJson
//      ↓
// Conversión
//      ↓
// Perfil UI
// ======================================================

import {
    Perfil,
    crearPerfil
} from "./core_perfil";


// ======================================================
// 🧠 PERFIL UI
// ======================================================

let perfilUi:
    Perfil =

        crearPerfil();


// ======================================================
// 📤 OBTENER PERFIL UI
// ======================================================

export function obtenerPerfilUi():

    Perfil

{

    return perfilUi;

}


// ======================================================
// 📥 ESTABLECER PERFIL UI
// ======================================================

export function establecerPerfilUi(

    perfil:
        Perfil

):

    void

{

    perfilUi =
        perfil;

}