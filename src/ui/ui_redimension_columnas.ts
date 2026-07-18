// ======================================================
// ↔️ ui_Redimension_Columnas RemapH V3
// ======================================================

import { COLUMNAS } from "./ui_columnas";

export const ANCHOS_DEFAULT={   //Ancho default al doble click en mover separador
    numero:42,
    estado:58,

    app:52,
    trigger:150,

    tipo:150,
    accion:150,
    ejecucion:150,

    color:52,
    nota:220
};

export const ANCHO_MINIMO=52;


export function activarRedimensionColumnas(
    cabecera:HTMLElement
):void{

    const celdas=
        cabecera.querySelectorAll<HTMLElement>(
            ".cabecera-celda"
        );


    celdas.forEach((celda,indice)=>{


        // Nota siempre ocupa espacio restante

        if(indice===celdas.length-1){
            return;
        }


        const divisor=
            document.createElement("div");


        divisor.className=
            "divisor-columna";


        celda.append(
            divisor
        );


        divisor.addEventListener(
            "mousedown",
            evento=>{

                evento.preventDefault();

                iniciarArrastre(
                    evento.clientX,
                    indice
                );

            }
        );


        divisor.addEventListener(
            "dblclick",
            ()=>{

                restaurarAncho(
                    indice
                );

            }
        );


    });

}


function obtenerVariable(
    indice:number
):string{


    return COLUMNAS[indice].ancho
        .replace("var(","")
        .replace(")","");

}


function iniciarArrastre(
    inicioX:number,
    indice:number
){


    const variable=
        obtenerVariable(
            indice
        );


    const estilos=
        getComputedStyle(
            document.documentElement
        );


    const anchoInicial=
        parseFloat(
            estilos.getPropertyValue(
                variable
            )
        );


    const mover=(evento:MouseEvent)=>{


        const nuevo=
            Math.max(

                ANCHO_MINIMO,

                anchoInicial+
                evento.clientX-
                inicioX

            );


        document.documentElement.style.setProperty(

            variable,

            `${nuevo}px`

        );

    };


    const soltar=()=>{


        window.removeEventListener(
            "mousemove",
            mover
        );


        window.removeEventListener(
            "mouseup",
            soltar
        );

    };


    window.addEventListener(
        "mousemove",
        mover
    );


    window.addEventListener(
        "mouseup",
        soltar
    );

}


function restaurarAncho(
    indice:number
){

    const columna=
        COLUMNAS[indice];


    const valor=
        ANCHOS_DEFAULT[
            columna.id as keyof typeof ANCHOS_DEFAULT
        ];


    if(!valor){
        return;
    }


    document.documentElement.style.setProperty(

        columna.ancho.replace("var(","").replace(")",""),

        `${valor}px`

    );

}