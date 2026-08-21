// ======================================================
// ↴ comp_Separador_Expandir
// ------------------------------------------------------
// Botón Expandir/Contraer de un header de Separadores.
// Vive en el carril de números (no dentro de .fila-separador),
// ocupando el slot que le correspondería al número en esa
// posición — ver nota de arquitectura de la Etapa B.
//
// El slot (.carril-expandir-slot) reserva SIEMPRE el mismo
// alto que una fila (var(--row-height)) y centra el botón
// adentro — el botón en sí conserva su alto natural de
// .ui-btn (28px). Sin este slot, el alto real del botón
// definía el alto de esa entrada del carril y desincronizaba
// la columna de números contra la tabla principal apenas se
// agregaba un separador.
// ======================================================

import type { SeparadorPerfil } from "../core/core_perfil";
import { crearBoton } from "./comp_boton";
import { reconstruirTabla } from "../ui/ui_tabla_control";

export function crearExpandirSeparador(
  separador: SeparadorPerfil,
  alModificar: () => void,
): HTMLElement {
  const slot = document.createElement("div");

  slot.className = "carril-expandir-slot";

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

  slot.append(boton);

  return slot;
}
