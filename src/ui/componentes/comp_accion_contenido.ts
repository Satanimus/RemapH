// ======================================================
// ⚙️ comp_Accion_Contenido
// ======================================================

import { crearBoton } from "./comp_boton";

import type { FilaPerfil } from "../../core/core_perfil";

import { textoAccionMultimedia } from "../../core/core_multimedia";

export function crearAccionTeclado(): HTMLButtonElement {
  return crearBoton({
    texto: "Capturar",
    clase: "capturador",
  });
}

export function crearAccionMultimedia(
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  return crearBoton({
    texto: textoAccionMultimedia(filaPerfil.accionReferencia),
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
