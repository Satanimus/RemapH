// ======================================================
// ⌨️🖱️ comp_Capturador RemapH V3
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";
import type { FilaPerfil } from "../../core/core_perfil";
import { crearBoton } from "./comp_boton";
import { reconstruirFila } from "../ui_tabla_control";
import {
    triggerATexto,
    triggerAHTML
} from "../../core/core_trigger";
import { abrirPopupModificador } from "./comp_popup_abrir";
import { iniciarCaptura } from "./comp_capturador_captura";

type DestinoCaptura =
    "Trigger" |
    "Accion";

export function crearCapturador(
    contexto:ContextoFila,
    filaPerfil:FilaPerfil,
    destino:DestinoCaptura="Trigger"
):HTMLButtonElement{

    const trigger =
        destino==="Trigger"
            ? filaPerfil.trigger
            : filaPerfil.accion;

    const tieneTrigger =
        trigger!==null &&
        trigger.gatillo!==null;

    const boton = crearBoton({
        texto:
            tieneTrigger
                ? triggerATexto(trigger)
                : "Capturar",

        html:
            tieneTrigger
                ? `
                    <div class="trigger-extra">+</div>
                    <div class="trigger-contenido">
                        ${triggerAHTML(trigger)}
                    </div>
                  `
                : "Capturar",

        clase:"capturador"
    });

    const botonExtra =
        boton.querySelector(
            ".trigger-extra"
        ) as HTMLDivElement|null;

    if(botonExtra){

        botonExtra.addEventListener(
            "click",
            evento=>{

                evento.stopPropagation();

                abrirPopupModificador(
                    evento,
                    contexto,
                    filaPerfil,
                    destino
                );

            }
        );

    }

    let capturando = false;

    boton.addEventListener(
        "click",
        ()=>{

            if(capturando){
                return;
            }

            capturando = true;
            boton.textContent = "Esperando...";

            iniciarCaptura(
                triggerCapturado=>{

                    if(!triggerCapturado){

                        capturando = false;

                        boton.innerHTML =
                            tieneTrigger
                                ? `
                                    <div class="trigger-extra">+</div>
                                    <div class="trigger-contenido">
                                        ${triggerAHTML(trigger!)}
                                    </div>
                                  `
                                : "Capturar";

                        return;

                    }

                    if(destino==="Trigger"){

                        filaPerfil.trigger =
                            triggerCapturado;

                    }else{

                        filaPerfil.accion =
                            triggerCapturado;

                    }

                    capturando = false;

                    reconstruirFila(
                        contexto.id
                    );

                }
            );

        }
    );

    return boton;

}