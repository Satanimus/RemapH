// ======================================================
// ↴ comp_Separador_Expandir
// ------------------------------------------------------
// Botón Expandir/Contraer de un header de Separadores.
// Vive en el carril de números (no dentro de .fila-grupo),
// ocupando el slot que le correspondería al número en esa
// posición — ver nota de arquitectura de la Etapa B.
//
// El slot (.carril-expandir-slot) reserva SIEMPRE el mismo
// alto que una fila (var(--row-height)) y centra el botón
// adentro — el botón en sí conserva su alto natural de
// .ui-btn (28px). Sin este slot, el alto real del botón
// definía el alto de esa entrada del carril y desincronizaba
// la columna de números contra la tabla principal apenas se
// agregaba un grupo.
// ======================================================

import type { SeparadorPerfil } from "../../core/core_perfil";
import { crearBoton } from "./comp_boton";
import { reconstruirTabla } from "../ui_tabla_control";

export function crearExpandirGrupo(
  grupo: SeparadorPerfil,
  alModificar: () => void,
): HTMLElement {
  const slot = document.createElement("div");

  slot.className = "carril-expandir-slot";

  slot.style.setProperty(
    "--grupo-color",
    grupo.color ? `var(--tag-${grupo.color})` : "var(--border-light)",
  );

  const boton = crearBoton({
    texto: grupo.expandido ? "↴" : "≫",
    titulo: grupo.expandido ? "Contraer" : "Expandir",
    clase: "carril-expandir",
  });

  boton.addEventListener("click", () => {
    grupo.expandido = !grupo.expandido;

    alModificar();

    reconstruirTabla();
  });

  slot.append(boton);

  return slot;
}
