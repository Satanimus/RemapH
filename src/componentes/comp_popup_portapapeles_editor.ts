// ======================================================
// 📋📝 comp_Popup_Portapapeles_Editor
// ------------------------------------------------------
// Editor de Portapapeles (filaPerfil.tipo === "portapapeles"),
// abierto al hacer clic en la columna Acción (ver comp_accion.ts).
//
// A diferencia de MenuExpress (comp_popup_menu_express_editor.ts),
// Portapapeles NO es dueño de ningún contenido propio (ver
// core_portapapeles.ts) — el único campo de Acción es el nombre de
// la ventana, que además es el título que muestra
// portapapeles_main.ts (barra superior) y el que back_portapapeles.rs
// usa para pintar el header. Sin lista de seleccionados/disponibles
// que armar acá.
//
// Mismo patrón persistente que el resto de popups Extra: se guarda
// al toque (sin botón guardar/cancelar) y se cierra solo al hacer
// clic afuera (mostrarPopup ya resuelve eso).
// ======================================================

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

export function abrirEditorPortapapeles(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  const portapapelesAccion = filaPerfil.portapapelesAccion;

  const popup = document.createElement("div");

  popup.className = "popup-extra popup-portapapeles-editor";

  popup.dataset.ayudaId = "portapapeles-editor";

  const inputNombre = document.createElement("input");

  inputNombre.type = "text";
  // Reusa el mismo estilo (ancho 100%) que el campo Nombre del
  // editor de MenuExpress — genérico, sin nada específico de menú.
  inputNombre.className = "popup-input popup-menu-editor-nombre";
  inputNombre.placeholder = "Nombre de la ventana";
  inputNombre.value = portapapelesAccion.nombre;

  inputNombre.addEventListener("input", () => {
    portapapelesAccion.nombre = inputNombre.value;

    // No se redibuja el popup entero acá (perdería el foco del input
    // mientras se escribe) — solo se refleja en la columna Acción de
    // la tabla, que sí puede reconstruirse en caliente. Mismo criterio
    // que el campo Nombre de comp_popup_menu_express_editor.ts.
    reconstruirFila(contexto.id);
    alModificar();
  });

  inputNombre.addEventListener("keydown", (evento) => {
    if (evento.key === "Enter") {
      ocultarPopup();
    }
  });

  popup.append(inputNombre);

  mostrarPopup(popup, evento.clientX, evento.clientY, () => {
    reconstruirFila(contexto.id);
  });

  inputNombre.focus();
}
