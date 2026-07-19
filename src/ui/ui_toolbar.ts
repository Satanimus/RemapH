// ======================================================
// 🧰 ui_Toolbar RemapH V3
// ------------------------------------------------------
// Barra superior principal.
//
// Estados del perfil (botón .perfil-estado):
//
// 🟢 Perfil Activo      → ui = json = caché
// 🔴 Perfil inactivo    → ui = json, caché vacía
// 🟡 Perfil editado, ¿guardar? → ui ≠ json
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

import {
    crearIndicador,
    actualizarIndicador
} from "./componentes/comp_indicador";


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
                Perfil Activo
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


    const cacheDot =
        crearIndicador("cache-dot");


    toolbar.querySelector(
        ".toolbar-left"

    )?.append(

        cacheDot

    );


    invoke<boolean>(

        "obtener_estado_cache"

    )

    .then(

        activo => {

            marcarPerfilSegunCache(

                toolbar,

                cacheDot,

                activo

            );

        }

    )

    .catch(

        error => {

            console.error(

                "❌ No se pudo obtener el estado de la caché:",

                error

            );

        }

    );


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

            const estadoActual =
                botonEstado.dataset.estado;


            botonEstado.disabled =
                true;


            try {

                if (estadoActual === "editado") {

                    await alGuardar();

                    const activo =
                        await invoke<boolean>(

                            "obtener_estado_cache"

                        );

                    marcarPerfilSegunCache(

                        toolbar,

                        cacheDot,

                        activo

                    );

                }

                else if (estadoActual === "activo") {

                    await invoke(

                        "desactivar_perfil"

                    );

                    marcarPerfilSegunCache(

                        toolbar,

                        cacheDot,

                        false

                    );

                }

                else if (estadoActual === "inactivo") {

                    const activo =
                        await invoke<boolean>(

                            "activar_perfil"

                        );

                    marcarPerfilSegunCache(

                        toolbar,

                        cacheDot,

                        activo

                    );

                }

            }

            catch(error) {

                console.error(

                    "❌ No se pudo cambiar el estado del perfil:",

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

                    botonEstado?.dataset.estado === "editado",

                    alGuardar,

                    resultado => {

                        aplicarResultadoPerfil(

                            toolbar,

                            botonSelector,

                            cacheDot,

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

    cacheDot:
        HTMLElement,

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


    marcarPerfilSegunCache(

        toolbar,

        cacheDot,

        resultado.cache_activo

    );

}


// ======================================================
// 🎯 MARCAR PERFIL SEGÚN CACHÉ
// ------------------------------------------------------
// Punto único que decide Activo/Inactivo + el color del
// indicador de caché, siempre en espejo (nunca pueden
// quedar desincronizados).
// ======================================================

function marcarPerfilSegunCache(

    toolbar:
        HTMLElement,

    cacheDot:
        HTMLElement,

    activo:
        boolean,

):

    void

{

    if (activo) {

        marcarPerfilActivo(

            toolbar

        );

    }

    else {

        marcarPerfilInactivo(

            toolbar

        );

    }


    actualizarIndicador(

        cacheDot,

        activo

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
        "Perfil editado, ¿guardar?";


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
        "Perfil Activo";


    botonEstado.dataset.estado =
        "activo";

}


// ======================================================
// 🔴 MARCAR PERFIL INACTIVO
// ======================================================

export function marcarPerfilInactivo(

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
        "Perfil inactivo";


    botonEstado.dataset.estado =
        "inactivo";

}