// ======================================================
// 🧩🎛️ comp_Popup_Macro_Extra
// ------------------------------------------------------
// Popup Extra del tipo "Macro" (filaPerfil.tipo === "macro"),
// abierto desde crearExtra() en comp_controles.ts. Mismo patrón
// persistente que el resto de los popups Extra propios (Abrir/
// Portapapeles/MenuExpress): elegir una opción actualiza
// filaPerfil.macroExtra y redibuja el mismo popup en el lugar, en
// vez de cerrarlo.
//
// A diferencia de esos popups, acá hay una sola sección: el
// Comportamiento (Una ejecución / Toggle / Tecla mantenida) — ver
// core_macro.ts para el detalle de qué significa cada uno. Desde
// la Etapa 8A, Extra dejó de ser la puerta de entrada al editor
// (eso vive ahora en Acción, ver comp_popup_macro_accion.ts).
// ======================================================

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui_tabla_control";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import type { ComportamientoMacro } from "../../core/core_macro";

import { crearGrupoOpciones, crearFilaPopup } from "./comp_popup_grupo";

// ======================================================
// 🧩🎛️ ABRIR POPUP EXTRA "MACRO"
// ======================================================

export function abrirPopupExtraMacro(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const macroExtra = filaPerfil.macroExtra;

  const dibujar = (): void => {
    const popup = document.createElement("div");

    popup.className = "popup-extra";

    const opciones: { texto: string; valor: ComportamientoMacro }[] = [
      { texto: "Una ejecución", valor: "una_ejecucion" },
      { texto: "Toggle", valor: "toggle" },
      { texto: "Tecla mantenida", valor: "tecla_mantenida" },
    ];

    popup.append(
      crearFilaPopup(
        "Comportamiento",
        crearGrupoOpciones(opciones, macroExtra.comportamiento, (valor) => {
          macroExtra.comportamiento = valor;

          reconstruirFila(contexto.id);
          dibujar();
        }),
      ),
    );

    mostrarPopup(popup, evento.clientX, evento.clientY);
  };

  dibujar();
}
