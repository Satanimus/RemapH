// ======================================================
// 📋 core_Perfil_Acciones RemapH V3
// ======================================================

import {
    obtenerPerfilUi
} from "./core_perfil_ui";

import {
    clonarFila
} from "./core_perfil";

import type {
    FilaPerfil
} from "./core_perfil";


// ======================================================
// 📋 CLONAR FILA POR ID
// ======================================================

export function clonarFilaPorId(

    id:
        string

):

    void

{

    const perfil =
        obtenerPerfilUi();


    const fila =
        perfil.filas.find(

            fila =>
                fila.id === id

        );


    if (!fila) {

        return;

    }


    perfil.filas.push(

        clonarFila(

            fila

        )

    );

}


// ======================================================
// 🗑️ ELIMINAR FILA POR ID
// ======================================================

export function eliminarFilaPorId(

    id:
        string

):

    void

{

    const perfil =
        obtenerPerfilUi();


    const indice =
        perfil.filas.findIndex(

            fila =>
                fila.id === id

        );


    if (indice < 0) {

        return;

    }


    perfil.filas.splice(

        indice,

        1

    );

}


// ======================================================
// ↕️ MOVER FILA POR ID
// ------------------------------------------------------
// Intercambia la fila con su vecina inmediata. No hace
// nada si ya está en el borde correspondiente.
// ======================================================

export function moverFilaPorId(

    id:
        string,

    direccion:
        "arriba" | "abajo"

):

    void

{

    const perfil =
        obtenerPerfilUi();


    const indice =
        perfil.filas.findIndex(

            fila =>
                fila.id === id

        );


    if (indice < 0) {

        return;

    }


    const destino =
        direccion === "arriba"
            ? indice - 1
            : indice + 1;


    if (

        destino < 0 ||
        destino >= perfil.filas.length

    ) {

        return;

    }


    const filas =
        perfil.filas;


    [filas[indice], filas[destino]] =
        [filas[destino], filas[indice]];

}


// ======================================================
// 🔍 ¿LA FILA TIENE ALGO EN ACCIÓN?
// ======================================================

export function filaTieneAccion(

    filaPerfil:
        FilaPerfil

):

    boolean

{

    return !!filaPerfil.accion?.gatillo;

}