// ======================================================
// 🎛️ comp_Controles RemapH V3
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";
import type { FilaPerfil } from "../../core/core_perfil";
import { crearPopup } from "./comp_popup";
import { reconstruirFila } from "../ui_tabla_control";

import {
    abrirPopupCondicion,
    abrirPopupTipo,
    abrirPopupApp,
    abrirPopupColor,
    abrirPopupEjecucion
} from "./comp_popup_abrir";

// ======================================================
// 🟢🔴 ESTADO (interruptor ON/OFF)
// ======================================================

export function crearEstado(
    contexto:ContextoFila,
    filaPerfil:FilaPerfil
):HTMLButtonElement{

    const boton=document.createElement("button");

    boton.className="ui-btn estado-toggle";

    boton.dataset.estado=
        filaPerfil.estado==="ON"?"on":"off";

    boton.textContent=filaPerfil.estado;

    boton.addEventListener(
        "click",
        ()=>{

            filaPerfil.estado=
                filaPerfil.estado==="ON"?"OFF":"ON";

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
    contexto:ContextoFila,
    filaPerfil:FilaPerfil
):HTMLButtonElement{

    return crearPopup({
        texto:filaPerfil.app,
        titulo:"Uso global",
        onClick:(evento)=>{

            abrirPopupApp(
                evento,
                contexto,
                filaPerfil
            );
        }
    });
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