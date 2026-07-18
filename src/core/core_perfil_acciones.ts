// ======================================================
// 📋 core_Perfil_Acciones RemapH V3
// ======================================================

import {
    obtenerPerfilUi
} from "./core_perfil_ui";

import {
    clonarFila
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