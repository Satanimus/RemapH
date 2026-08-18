// ======================================================
// ▼ comp_Popup
// ------------------------------------------------------
// Ya no antepone la flecha "▾" al texto — la columna Extra
// (único consumidor de este helper) queda con el texto solo,
// mismo criterio que la columna Tipo (ver comp_popup_abrir.ts,
// iconoDeTipo) y App (ver crearApp en este mismo archivo).
// ======================================================

import { crearBoton } from "./comp_boton";

export interface PopupOpciones {
  texto: string;
  titulo?: string;
  onClick?: (evento: MouseEvent, actualizar: (texto: string) => void) => void;
}

export function crearPopup(opciones: PopupOpciones): HTMLButtonElement {
  const boton = crearBoton({
    texto: opciones.texto,
    titulo: opciones.titulo,
  });

  if (opciones.onClick) {
    boton.addEventListener("click", (evento) => {
      opciones.onClick!(evento, (texto: string) => {
        boton.textContent = texto;
      });
    });
  }

  return boton;
}
