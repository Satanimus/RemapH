// ======================================================
// 🪟 comp_Popup_Abrir RemapH V3
// ======================================================

import {
    mostrarPopup,
    ocultarPopup
} from "./comp_popup_contenedor";

import type { ContextoFila } from "../../core/core_contexto_fila";
import { clonarFilaPorId, eliminarFilaPorId, filaTieneAccion } from "../../core/core_perfil_acciones";
import type { FilaPerfil } from "../../core/core_perfil";
import { crearEntrada } from "../../core/core_entrada";
import { reconstruirFila } from "../ui_tabla_control";
import { reconstruirTabla } from "../ui_tabla_control";
import { activarModoMover } from "../ui_tabla_control";

function crearLista(
    opciones: string[],
    seleccion?: (
        valor: string,
    ) => void,
): HTMLElement {

    const lista =
        document.createElement(
            "div",
        );

    lista.className =
        "popup-lista";

    opciones.forEach(
        opcion => {

            const boton =
                document.createElement(
                    "button",
                );

            boton.className =
                "ui-btn";

            boton.textContent =
                opcion;

            boton.addEventListener(
                "click",
                () => {

                    if (
                        seleccion
                    ) {

                        seleccion(
                            opcion,
                        );
                    }

                    ocultarPopup();
                },
            );

            lista.append(
                boton,
            );
        },
    );

    return lista;
}

function abrirLista(
    evento:MouseEvent,
    opciones:string[],
    actualizar:(texto:string)=>void
):void{

    mostrarPopup(
        crearLista(
            opciones,
            actualizar
        ),
        evento.clientX,
        evento.clientY
    );

}

export function abrirPopupCondicion(
    evento:MouseEvent,
    actualizar:(texto:string)=>void
):void{

    abrirLista(
        evento,
        [
            "Normal",
            "Mantener pulsado",
            "Doble toque"
        ],
        actualizar
    );

}

export function abrirPopupTipo(
    evento:MouseEvent,
    actualizar:(texto:string)=>void,
    _contexto:ContextoFila
):void{

    abrirLista(
        evento,
        [
            "Teclado",
            "Mouse",
            "Click coordenada",
            "Multimedia",
            "Macro",
            "Portapapeles"
        ],
        actualizar
    );

}

export function abrirPopupEstado(
    evento:MouseEvent,
    actualizar:(texto:string)=>void,
    contexto:ContextoFila
):void{

    abrirLista(
        evento,
        [
            "ON",
            "OFF",
            "Clonar",
            "Eliminar"
        ],
        (texto)=>{

            if(
                texto==="ON" ||
                texto==="OFF"
            ){

                actualizar(texto);

            }

            if(
                texto==="Clonar"
            ){

                clonarFilaPorId(
                    contexto.id
                );

                reconstruirTabla();

            }

        }
    );

}

// ======================================================
// 🔢 POPUP NÚMERO DE FILA
// ------------------------------------------------------
// Mover / Clonar / Eliminar. No usa abrirLista porque
// Eliminar necesita doble confirmación in-place, y las
// tres acciones modifican el perfil (marcan editado).
// ======================================================

export function abrirPopupNumero(
    evento:MouseEvent,
    contexto:ContextoFila,
    filaPerfil:FilaPerfil,
    alModificar:()=>void
):void{

    const lista=document.createElement("div");

    lista.className="popup-lista";

    // ----------------------------------
    // ↕️ MOVER
    // ----------------------------------

    const botonMover=document.createElement("button");

    botonMover.className="ui-btn";
    botonMover.textContent="Mover";

    botonMover.addEventListener(
        "click",
        ()=>{

            activarModoMover();
            alModificar();
            reconstruirTabla();
            ocultarPopup();

        }
    );

    // ----------------------------------
    // 📋 CLONAR
    // ----------------------------------

    const botonClonar=document.createElement("button");

    botonClonar.className="ui-btn";
    botonClonar.textContent="Clonar";

    botonClonar.addEventListener(
        "click",
        ()=>{

            clonarFilaPorId(contexto.id);
            alModificar();
            reconstruirTabla();
            ocultarPopup();

        }
    );

    // ----------------------------------
    // 🗑️ ELIMINAR
    // ----------------------------------

    const botonEliminar=document.createElement("button");

    botonEliminar.className="ui-btn popup-perfil-eliminar";
    botonEliminar.textContent="Eliminar";

    let confirmando=false;

    botonEliminar.addEventListener(
        "click",
        ()=>{

            if(filaTieneAccion(filaPerfil) && !confirmando){

                confirmando=true;

                botonEliminar.textContent=
                    "⚠️ Confirmar eliminación";

                return;

            }

            eliminarFilaPorId(contexto.id);
            alModificar();
            reconstruirTabla();
            ocultarPopup();

        }
    );

    lista.append(
        botonMover,
        botonClonar,
        botonEliminar
    );

    mostrarPopup(
        lista,
        evento.clientX,
        evento.clientY
    );

}

export function abrirPopupApp(
    evento:MouseEvent,
    contexto:ContextoFila,
    filaPerfil:FilaPerfil
):void{

    abrirLista(
        evento,
        [
            "🌐",
            "📝 Word.exe",
            "🎨 Photoshop.exe"
        ],
        (texto)=>{

            filaPerfil.app=texto;

            reconstruirFila(
                contexto.id
            );

        }
    );

}

export function abrirPopupColor(
    evento:MouseEvent,
    contexto:ContextoFila,
    filaPerfil:FilaPerfil
):void{

    abrirLista(
        evento,
        [
            "🔴",
            "🟢",
            "🔵",
            "🟡"
        ],
        (texto)=>{

            filaPerfil.color=texto;

            reconstruirFila(
                contexto.id
            );

        }
    );

}

export function abrirPopupEjecucion(
    evento:MouseEvent,
    contexto:ContextoFila,
    filaPerfil:FilaPerfil
):void{

    abrirLista(
        evento,
        [
            "Normal",
            "Turbo",
            "Mantener"
        ],
        (texto)=>{

            filaPerfil.ejecucion=texto;

            reconstruirFila(
                contexto.id
            );

        }
    );

}

export function abrirPopupModificador(
    evento:MouseEvent,
    contexto:ContextoFila,
    filaPerfil:FilaPerfil,
    destino:"Trigger"|"Accion"="Trigger"
):void{

    abrirLista(
        evento,
        [
            "Win +",
            "Ctrl Izq +",
            "Shift Izq +",
            "Alt Izq +"
        ],
        (texto)=>{

            const entrada=
                crearModificador(
                    texto
                );

            if(!entrada){
                return;
            }

            const trigger=
                destino==="Trigger"
                ?filaPerfil.trigger
                :filaPerfil.accion;

            if(!trigger){
                return;
            }

            const existe=
                trigger.modificadores.some(
                    modificador=>
                        modificador.codigo===
                        entrada.codigo
                );

            if(existe){
                return;
            }

            trigger.modificadores.unshift(
                entrada
            );

            reconstruirFila(
                contexto.id
            );

        }
    );

}

function crearModificador(
    texto:string
){

    switch(texto){

        case "Win +":

            return crearEntrada(
                "Teclado",
                "MetaLeft",
                "Meta"
            );

        case "Ctrl Izq +":

            return crearEntrada(
                "Teclado",
                "ControlLeft",
                "Control"
            );

        case "Shift Izq +":

            return crearEntrada(
                "Teclado",
                "ShiftLeft",
                "Shift"
            );

        case "Alt Izq +":

            return crearEntrada(
                "Teclado",
                "AltLeft",
                "Alt"
            );

        default:

            return null;

    }

}