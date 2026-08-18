// ======================================================
// ↴ comp_Grupo_Expandir
// ------------------------------------------------------
// Botón Expandir/Contraer de un header de Agrupación.
// Vive en el carril de números (no dentro de .fila-grupo),
// ocupando el slot que le correspondería al número en esa
// posición — ver nota de arquitectura de la Etapa B.
// ======================================================

import type { AgrupacionPerfil } from "../../core/core_perfil";
import { crearBoton } from "./comp_boton";
import { reconstruirTabla } from "../ui_tabla_control";

export function crearExpandirGrupo(
  grupo: AgrupacionPerfil,
  alModificar: () => void,
): HTMLElement {
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

  return boton;
}
