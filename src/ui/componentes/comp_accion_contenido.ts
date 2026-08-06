// ======================================================
// ⚙️ comp_Accion_Contenido
// ======================================================

import { crearBoton } from "./comp_boton";

import type { FilaPerfil } from "../../core/core_perfil";

import { textoAccionMultimedia } from "../../core/core_multimedia";

import { textoMenuAccion } from "../../core/core_menu_express";

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

// El editor propio (seleccionados/disponibles) se conecta recién en
// la Etapa 3 — por ahora el botón solo muestra el nombre del menú
// (o el default), sin abrir nada al hacer clic (ver comp_accion.ts).
export function crearAccionMenuExpress(
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  return crearBoton({
    texto: textoMenuAccion(filaPerfil.menuAccion),
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
