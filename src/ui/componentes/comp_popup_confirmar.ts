// ======================================================
// ❓ comp_Popup_Confirmar RemapH V3
// ------------------------------------------------------
// Diálogo genérico Sí/No, reutilizable.
//
// Se resuelve con true (Sí), false (No), y también false
// si se cierra clickeando afuera (equivale a "No").
// ======================================================

import {
    mostrarPopup,
    ocultarPopup
} from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

export function confirmarPopup(
    mensaje:string,
    evento:MouseEvent
):Promise<boolean>{

    return new Promise(resolver=>{

        let resuelto=false;

        const resolverUnaVez=(valor:boolean)=>{

            if(resuelto){
                return;
            }

            resuelto=true;

            resolver(valor);

        };

        const contenedor=document.createElement("div");

        contenedor.className="popup-confirmar";

        const texto=document.createElement("p");

        texto.className="popup-confirmar-mensaje";
        texto.textContent=mensaje;

        const botones=document.createElement("div");

        botones.className="popup-confirmar-botones";

        const botonNo=crearBoton({ texto:"No" });
        const botonSi=crearBoton({ texto:"Sí" });

        botonNo.addEventListener(
            "click",
            ()=>{
                resolverUnaVez(false);
                ocultarPopup();
            }
        );

        botonSi.addEventListener(
            "click",
            ()=>{
                resolverUnaVez(true);
                ocultarPopup();
            }
        );

        botones.append(botonNo,botonSi);
        contenedor.append(texto,botones);

        mostrarPopup(
            contenedor,
            evento.clientX,
            evento.clientY,
            ()=>resolverUnaVez(false)
        );

    });

}