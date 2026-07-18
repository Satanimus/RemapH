// ======================================================
// ▼ comp_Popup RemapH V3
// ======================================================

import { crearBoton } from "./comp_boton";

export interface PopupOpciones{
    texto:string;
    titulo?:string;
    onClick?:(
        evento:MouseEvent,
        actualizar:(texto:string)=>void
    )=>void;
}

export function crearPopup(
    opciones:PopupOpciones
):HTMLButtonElement{

    const boton=crearBoton({
        texto:`${opciones.texto} ▾`,
        titulo:opciones.titulo
    });

    if(opciones.onClick){

        boton.addEventListener(
            "click",
            (evento)=>{

                opciones.onClick!(
                    evento,
                    (texto:string)=>{

                        boton.textContent=
                            `${texto} ▾`;

                    }
                );

            }
        );

    }

    return boton;

}