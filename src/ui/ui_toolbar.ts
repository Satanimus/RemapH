// ======================================================
// 🧰 ui_Toolbar RemapH V3
// ------------------------------------------------------
// Barra superior principal.
//
// Estados del perfil:
//
// 🟢 PERFIL ACTIVO
// 🟡 PERFIL EDITADO
// 🔴 PERFIL PAUSADO
// ======================================================

import {
    invoke
} from "@tauri-apps/api/core";

import {
    abrirPopupPerfil
} from "./componentes/comp_popup_perfil";

import type {
    ResultadoPerfil
} from "./componentes/comp_popup_perfil";

import {
    convertirPerfilJson
} from "../core/core_perfil_json";

import {
    establecerPerfilUi
} from "../core/core_perfil_ui";

import {
    reconstruirTabla
} from "./ui_tabla_control";


// ======================================================
// 🚀 CREAR TOOLBAR
// ======================================================

export function crearToolbar(

    alCrearFila:
        () => void,

    alGuardar:
        () => Promise<void>

):

    HTMLElement

{

    const toolbar =
        document.createElement(

            "header"

        );


    toolbar.className =
        "toolbar";


    toolbar.innerHTML = `

        <div class="toolbar-left">

            <div class="titulo">
                RemapH V3
            </div>

        </div>


        <div class="toolbar-center">

            <button class="perfil-selector">
                Default ▾
            </button>


            <button class="perfil-estado">
                PERFIL ACTIVO
            </button>

        </div>


        <div class="toolbar-right">

            <button class="btn-nueva-fila">
                + Fila
            </button>


            <button class="configuracion">
                ⚙
            </button>

        </div>

    `;


    const botonNuevaFila =
        toolbar.querySelector<HTMLButtonElement>(

            ".btn-nueva-fila"

        );


    botonNuevaFila?.addEventListener(

        "click",

        () => {

            alCrearFila();

        }

    );


    const botonEstado =
        toolbar.querySelector<HTMLButtonElement>(

            ".perfil-estado"

        );


    botonEstado?.addEventListener(

        "click",

        async () => {

            if (

                botonEstado.dataset.estado !==
                "editado"

            ) {

                return;

            }


            botonEstado.disabled =
                true;


            try {

                await alGuardar();


                marcarPerfilActivo(

                    toolbar

                );

            }

            catch(error) {

                console.error(

                    "❌ No se pudo guardar el perfil:",

                    error

                );

            }

            finally {

                botonEstado.disabled =
                    false;

            }

        }

    );


    const botonSelector =
        toolbar.querySelector<HTMLButtonElement>(

            ".perfil-selector"

        );


    if (botonSelector) {

        invoke<string>(

            "obtener_nombre_perfil_actual"

        )

        .then(

            nombre => {

                botonSelector.textContent =
                    `${nombre} ▾`;

            }

        )

        .catch(

            error => {

                console.error(

                    "❌ No se pudo obtener el perfil actual:",

                    error

                );

            }

        );


        botonSelector.addEventListener(

            "click",

            evento => {

                abrirPopupPerfil(

                    evento,

                    nombreMostrado(

                        botonSelector

                    ),

                    resultado => {

                        aplicarResultadoPerfil(

                            toolbar,

                            botonSelector,

                            resultado,

                        );

                    },

                );

            }

        );

    }


    return toolbar;

}


// ======================================================
// 🆔 NOMBRE MOSTRADO EN EL SELECTOR
// ======================================================

function nombreMostrado(

    botonSelector:
        HTMLButtonElement,

):

    string

{

    return (

        botonSelector.textContent ??
            ""

    )

        .replace(

            "▾",

            "",

        )

        .trim();

}


// ======================================================
// 🔄 APLICAR RESULTADO DE PERFIL
// ------------------------------------------------------
// Reemplaza el perfil de la UI con el que llega desde
// Rust (seleccionar / crear / clonar / renombrar /
// eliminar perfil) y refresca toolbar + tabla.
// ======================================================

function aplicarResultadoPerfil(

    toolbar:
        HTMLElement,

    botonSelector:
        HTMLButtonElement,

    resultado:
        ResultadoPerfil,

):

    void

{

    const perfil =

        convertirPerfilJson(

            resultado.perfil

        );


    establecerPerfilUi(

        perfil

    );


    reconstruirTabla();


    botonSelector.textContent =
        `${resultado.nombre} ▾`;


    marcarPerfilActivo(

        toolbar

    );

}


// ======================================================
// 🟡 MARCAR PERFIL EDITADO
// ======================================================

export function marcarPerfilEditado(

    toolbar:
        HTMLElement

):

    void

{

    const botonEstado =
        toolbar.querySelector<HTMLButtonElement>(

            ".perfil-estado"

        );


    if (!botonEstado) {

        return;

    }


    botonEstado.textContent =
        "PERFIL EDITADO";


    botonEstado.dataset.estado =
        "editado";

}


// ======================================================
// 🟢 MARCAR PERFIL ACTIVO
// ======================================================

export function marcarPerfilActivo(

    toolbar:
        HTMLElement

):

    void

{

    const botonEstado =
        toolbar.querySelector<HTMLButtonElement>(

            ".perfil-estado"

        );


    if (!botonEstado) {

        return;

    }


    botonEstado.textContent =
        "PERFIL ACTIVO";


    botonEstado.dataset.estado =
        "activo";

}