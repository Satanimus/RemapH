// ======================================================
// ⚙️ comp_Accion
//
// ======================================================

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import {
  crearAccionMultimedia,
  crearAccionMenuExpress,
  crearAccionMacro,
  crearAccionPortapapeles,
} from "./comp_accion_contenido";

import { crearAccionAbrir } from "./comp_popup_abrir_accion";

import { crearCapturador } from "./comp_capturador";

import { abrirPopupAccionMultimedia } from "./comp_popup_multimedia";

import { abrirEditorMenuExpress } from "./comp_popup_menu_express_editor";

import { abrirEditorPortapapeles } from "./comp_popup_portapapeles_editor";

import { abrirPopupMacroAccion } from "./comp_popup_macro_accion";

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
        abrirPopupAccionMultimedia(evento, contexto, filaPerfil, alModificar);
      });

      return boton;
    }

    // El clic abre el editor (seleccionados/disponibles) — ver
    // comp_popup_menu_express_editor.ts.
    case "menu_express": {
      const boton = crearAccionMenuExpress(filaPerfil);

      boton.addEventListener("click", (evento) => {
        abrirEditorMenuExpress(evento, contexto, filaPerfil, alModificar);
      });

      return boton;
    }

    // El clic abre el menú completo (Editar/Renombrar/Nueva/Abrir/
    // Clonar/Eliminar) — ver comp_popup_macro_accion.ts.
    case "macro": {
      const boton = crearAccionMacro(filaPerfil);

      boton.addEventListener("click", (evento) => {
        abrirPopupMacroAccion(evento, contexto, filaPerfil, alModificar);
      });

      return boton;
    }

    // El clic abre el editor (solo Nombre de la ventana) — ver
    // comp_popup_portapapeles_editor.ts.
    case "portapapeles": {
      const boton = crearAccionPortapapeles(filaPerfil);

      boton.addEventListener("click", (evento) => {
        abrirEditorPortapapeles(evento, contexto, filaPerfil, alModificar);
      });

      return boton;
    }

    // El botón (ícono + nombre + tooltip) y el popup de selección
    // Archivo/Carpeta viven juntos en comp_popup_abrir_accion.ts —
    // a diferencia de los casos de arriba, acá alcanza con un solo
    // llamado (mismo criterio que crearCapturador() en el default).
    case "abrir":
      return crearAccionAbrir(contexto, filaPerfil, alModificar);

    // El extra Coordenada (dentro de Tecla/Mouse) no tiene un botón
    // propio: reusa este mismo capturador — lo capturado es lo que se
    // ejecuta en la coordenada calculada (ver popup Extra).
    default:
      return crearCapturador(contexto, filaPerfil, "Accion", alModificar);
  }
}
