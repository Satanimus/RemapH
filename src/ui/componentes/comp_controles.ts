// ======================================================
// 🎛️ comp_Controles RemapH V3
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";
import type { FilaPerfil } from "../../core/core_perfil";
import { crearPopup } from "./comp_popup";
import { reconstruirFila } from "../ui_tabla_control";
import { invoke } from "@tauri-apps/api/core";

import {
    abrirPopupCondicion,
    abrirPopupTipo,
    abrirPopupColor,
    abrirPopupEjecucion
} from "./comp_popup_abrir";

import {
    abrirPopupApp
} from "./comp_popup_app";

import {
    obtenerPerfilUi
} from "../../core/core_perfil_ui";

import {
    filaTieneConflicto
} from "../../core/core_conflictos";

// ======================================================
// 🟢🔴 ESTADO (interruptor ON/OFF)
// ======================================================

export function crearEstado(

    contexto:
        ContextoFila,

    filaPerfil:
        FilaPerfil

):

    HTMLButtonElement

{

    const boton =
        document.createElement(

            "button"

        );


    boton.className =
        "ui-btn estado-toggle";


    const conflicto =
        filaTieneConflicto(

            filaPerfil.id,

            obtenerPerfilUi().filas

        );


    boton.dataset.estado =

        conflicto

        ?

        "off"

        :

        filaPerfil.estado === "ON"

        ?

        "on"

        :

        "off";


    boton.dataset.conflicto =

        conflicto

        ?

        "true"

        :

        "false";


    const texto =
        document.createElement(

            "span"

        );


    texto.textContent =

        conflicto

        ?

        "OFF"

        :

        filaPerfil.estado;


    boton.append(

        texto

    );


    if (

        conflicto

    ) {

        const alerta =
            document.createElement(

                "span"

            );

        alerta.className =
            "estado-alerta";

        alerta.textContent =
            "⚠";


        boton.append(

            alerta

        );

    }


    boton.addEventListener(

        "click",

        evento => {

            if (

                conflicto

            ) {

                evento.stopPropagation();

                return;

            }


            filaPerfil.estado =

                filaPerfil.estado === "ON"

                ?

                "OFF"

                :

                "ON";


            reconstruirFila(

                contexto.id

            );

        }

    );


    return boton;

}

export function crearCondicion(
    contexto:ContextoFila,
    filaPerfil:FilaPerfil
):HTMLButtonElement{

    return crearPopup({
        texto:filaPerfil.condicion,
        onClick:(evento)=>{

            abrirPopupCondicion(
                evento,
                (texto)=>{

                    filaPerfil.condicion=texto;

                    reconstruirFila(
                        contexto.id
                    );

                }
            );

        }
    });

}

export function crearTipo(
    contexto:ContextoFila,
    filaPerfil:FilaPerfil
):HTMLButtonElement{

    return crearPopup({
        texto:filaPerfil.tipo,
        onClick:(evento)=>{

            abrirPopupTipo(
                evento,
                (texto)=>{

                    filaPerfil.tipo=texto;

                    reconstruirFila(
                        contexto.id
                    );

                },
                contexto
            );
        }
    });
}

export function crearNota(
    filaPerfil:FilaPerfil
):HTMLInputElement{

    const input=document.createElement("input");

    input.className="nota";

    input.placeholder="Nota...";

    input.value=filaPerfil.nota;

    input.addEventListener(
        "input",
        ()=>{

            filaPerfil.nota=input.value;
        }
    );

    return input;
}

export function crearEjecucion(
    contexto:ContextoFila,
    filaPerfil:FilaPerfil
):HTMLButtonElement{

    return crearPopup({
        texto:filaPerfil.ejecucion,
        onClick:(evento)=>{

            abrirPopupEjecucion(
                evento,
                contexto,
                filaPerfil
            );
        }
    });
}

export function crearApp(

    contexto:
        ContextoFila,

    filaPerfil:
        FilaPerfil

):

    HTMLButtonElement

{

    const boton =
        document.createElement(
            "button"
        );

    boton.className =
        "ui-btn app-control";

    boton.title =
        filaPerfil.app.programa
        ??
        "Uso global";

    const icono =
        document.createElement(
            "span"
        );

    icono.className =
        "app-icono-fallback";

    icono.textContent =
        filaPerfil.app.programa
        ?

        "▣"

        :

        "🌐";

    boton.append(

        icono

    );

    if (

        filaPerfil.app.segundoPlano

    ) {

        const indicador =
            document.createElement(
                "span"
            );

        indicador.className =
            "app-segundo-plano-indicador";

        indicador.textContent =
            "∶";

        boton.append(

            indicador

        );

    }

    const flecha =
        document.createElement(
            "span"
        );

    flecha.className =
        "app-flecha";

    flecha.textContent =
        "▾";

    boton.append(

        flecha

    );

    if (

        filaPerfil.app.programa

    ) {

        invoke<{

            ancho:
                number;

            alto:
                number;

            pixeles:
                string;

        } | null>(

            "obtener_icono_programa",

            {

                nombre:
                    filaPerfil.app.programa

            }

        )

            .then(

                iconoJson => {

                    if (!iconoJson) {

                        return;

                    }

                    const canvas =
                        document.createElement(
                            "canvas"
                        );

                    canvas.width =
                        iconoJson.ancho;

                    canvas.height =
                        iconoJson.alto;

                    const contextoCanvas =
                        canvas.getContext(
                            "2d"
                        );

                    if (!contextoCanvas) {

                        return;

                    }

                    const pixeles =
                        Uint8ClampedArray.from(

                            atob(

                                iconoJson.pixeles

                            ),

                            caracter =>
                                caracter.charCodeAt(0)

                        );

                    contextoCanvas.putImageData(

                        new ImageData(

                            pixeles,

                            iconoJson.ancho,

                            iconoJson.alto

                        ),

                        0,

                        0

                    );

                    canvas.className =
                        "app-icono";

                    boton.replaceChild(

                        canvas,

                        icono

                    );

                }

            )

            .catch(

                () => {}

            );

    }

    boton.addEventListener(

        "click",

        evento => {

            abrirPopupApp(

                evento,

                contexto,

                filaPerfil

            );

        }

    );

    return boton;

}

export function crearColor(
    contexto:ContextoFila,
    filaPerfil:FilaPerfil
):HTMLButtonElement{

    return crearPopup({
        texto:filaPerfil.color || "🎨",
        titulo:"Color",
        onClick:(evento)=>{

            abrirPopupColor(
                evento,
                contexto,
                filaPerfil
            );
        }
    });
}