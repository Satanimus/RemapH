// ======================================================
// ⚙️ comp_Accion
//
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import {
  crearAccionMultimedia,
  crearAccionMenuExpress,
  crearAccionMacro,
  crearAccionPortapapeles,
} from "./comp_accion_contenido";

import { crearCapturador } from "./comp_capturador";

import { abrirPopupAccionMultimedia } from "./comp_popup_multimedia";

import { abrirEditorMenuExpress } from "./comp_popup_menu_express_editor";

// ======================================================
// CREAR ACCIÓN
// ======================================================

export function crearAccion(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): HTMLButtonElement {
  switch (filaPerfil.tipo) {
    case "multimedia": {
      const boton = crearAccionMultimedia(filaPerfil);

      boton.addEventListener("click", (evento) => {
        abrirPopupAccionMultimedia(evento, contexto, filaPerfil);

        alModificar();
      });

      return boton;
    }

    // El clic abre el editor (seleccionados/disponibles) — ver
    // comp_popup_menu_express_editor.ts.
    case "menu_express": {
      const boton = crearAccionMenuExpress(filaPerfil);

      boton.addEventListener("click", (evento) => {
        abrirEditorMenuExpress(evento, contexto, filaPerfil);

        alModificar();
      });

      return boton;
    }

    case "macro":
      return crearAccionMacro();

    case "portapapeles":
      return crearAccionPortapapeles();

    // El extra Coordenada (dentro de Tecla/Mouse) no tiene un botón
    // propio: reusa este mismo capturador — lo capturado es lo que se
    // ejecuta en la coordenada calculada (ver popup Extra).
    default:
      return crearCapturador(contexto, filaPerfil, "Accion", alModificar);
  }
}
