// ======================================================
// ↴ comp_Separador_Expandir
// ------------------------------------------------------
// Botón Expandir/Contraer de un header de Separadores, más
// el botón On/Off del separador (Etapa D) a su izquierda,
// lado a lado dentro del mismo slot. Vive en el carril de
// números (no dentro de .fila-separador) — ver nota de
// arquitectura de la Etapa B.
//
// El slot (.carril-expandir-slot) reserva SIEMPRE el mismo
// alto que una fila (var(--row-height)) y centra los botones
// adentro — cada botón conserva su alto natural de .ui-btn.
// Sin este slot, el alto real de los botones definía el alto
// de esa entrada del carril y desincronizaba la columna de
// números contra la tabla principal apenas se agregaba un
// separador.
// ======================================================

import type { SeparadorPerfil } from "../core/core_perfil";
import { crearBoton } from "./comp_boton";
import { reconstruirTabla } from "../ui/ui_tabla_control";

export function crearExpandirSeparador(
  separador: SeparadorPerfil,
  alModificar: () => void,
  botonEstado: HTMLElement,
): HTMLElement {
  const slot = document.createElement("div");

  slot.className = "carril-expandir-slot";

  slot.dataset.id = separador.id;

  slot.style.setProperty(
    "--separador-color",
    separador.color ? `var(--tag-${separador.color})` : "var(--border-light)",
  );

  const boton = crearBoton({
    texto: separador.expandido ? "↴" : "≫",
    titulo: separador.expandido ? "Contraer" : "Expandir",
    clase: "carril-expandir",
  });

  boton.addEventListener("click", () => {
    separador.expandido = !separador.expandido;

    alModificar();

    reconstruirTabla();
  });

  slot.append(botonEstado, boton);

  return slot;
}
