// ======================================================
// ⚙️ comp_Accion_Contenido
// ======================================================

import { crearBoton } from "./comp_boton";

export function crearAccionTeclado(): HTMLButtonElement {
  return crearBoton({
    texto: "Capturar",
    clase: "capturador",
  });
}

export function crearAccionMultimedia(): HTMLButtonElement {
  return crearBoton({
    texto: "Multimedia",
    clase: "capturador",
  });
}

export function crearAccionMacro(): HTMLButtonElement {
  return crearBoton({
    texto: "Macro",
    clase: "capturador",
  });
}

export function crearAccionPortapapeles(): HTMLButtonElement {
  return crearBoton({
    texto: "Portapapeles",
    clase: "capturador",
  });
}
