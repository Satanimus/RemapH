// ======================================================
// 🏗️ ui_Layout RemapH V3
// ======================================================

import {
    crearToolbar,
    marcarPerfilEditado
} from "./ui_toolbar";

import {
    crearTabla
} from "./ui_tabla";

import {
    crearStatusbar
} from "./ui_statusbar";

import {
    crearContenedorPopup
} from "./componentes/comp_popup_contenedor";

import {
    obtenerPerfilUi
} from "../core/core_perfil_ui";

import {
    reconstruirTabla
} from "./ui_tabla_control";

import {
    crearFila
} from "../core/core_perfil";


// ======================================================
// 🚀 CREAR LAYOUT
// ======================================================

export function crearLayout(

    alGuardar:
        () => Promise<void>

):

    HTMLElement

{

    const toolbar =
        crearToolbar(

            () => {

                const perfil =
                    obtenerPerfilUi();


                perfil.filas.push(

                    crearFila()

                );


                reconstruirTabla();


                marcarPerfilEditado(

                    toolbar

                );

            },

            alGuardar

        );


    const tabla =
        crearTabla(

            () => {

                marcarPerfilEditado(

                    toolbar

                );

            }

        );


    const fragment =
        document.createDocumentFragment();


    fragment.append(

        toolbar,

        tabla,

        crearStatusbar(),

        crearContenedorPopup()

    );


    const contenedor =
        document.createElement(

            "div"

        );


    contenedor.className =
        "layout";


    contenedor.append(

        fragment

    );


    return contenedor;

}