// ======================================================
// ⚙️ comp_Accion
//
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import {
  crearAccionMultimedia,
  crearAccionMacro,
  crearAccionPortapapeles,
} from "./comp_accion_contenido";

import { crearCapturador } from "./comp_capturador";

// ======================================================
// CREAR ACCIÓN
// ======================================================

export function crearAccion(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): HTMLButtonElement {
  switch (filaPerfil.tipo) {
    case "multimedia":
      return crearAccionMultimedia();

    case "macro":
      return crearAccionMacro();

    case "portapapeles":
      return crearAccionPortapapeles();

    // click_coordenada NO tiene un botón propio: reusa el mismo
    // capturador de Tecla/Mouse (ver default) — lo capturado ahí es
    // lo que se ejecuta en la coordenada calculada (ver popup Extra).
    default:
      return crearCapturador(contexto, filaPerfil, "Accion", alModificar);
  }
}
