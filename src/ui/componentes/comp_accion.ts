// ======================================================
// ⚙️ comp_Accion
//
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import {
  crearAccionMultimedia,
  crearAccionMacro,
  crearAccionCoordenada,
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

    case "click_coordenada":
      return crearAccionCoordenada();

    case "portapapeles":
      return crearAccionPortapapeles();

    default:
      return crearCapturador(contexto, filaPerfil, "Accion", alModificar);
  }
}
