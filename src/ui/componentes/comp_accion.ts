// ======================================================
// ⚙️ comp_Accion
// RemapH V3
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import {
  crearAccionMultimedia,
  crearAccionMacro,
  crearAccionCoordenada,
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
    case "Multimedia":
      return crearAccionMultimedia();

    case "Macro":
      return crearAccionMacro();

    case "Click coordenada":
      return crearAccionCoordenada();

    default:
      return crearCapturador(contexto, filaPerfil, "Accion", alModificar);
  }
}
