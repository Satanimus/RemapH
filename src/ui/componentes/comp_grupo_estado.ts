// ======================================================
// 🟢🔴 comp_Grupo_Estado
// ------------------------------------------------------
// Botón Estado del header de una Agrupación. Versión mínima:
// en esta etapa solo alterna grupo.estado — el forzado sobre
// las filas contenidas y el indicador gris de "ya no vigente"
// se agregan acá mismo en la Etapa E, sin rehacer el componente.
// ======================================================

import type { AgrupacionPerfil } from "../../core/core_perfil";
import { reconstruirTabla } from "../ui_tabla_control";

export function crearEstadoGrupo(
  grupo: AgrupacionPerfil,
  alModificar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn estado-toggle";

  boton.dataset.estado = grupo.estado === "ON" ? "on" : "off";

  const texto = document.createElement("span");

  texto.textContent = grupo.estado;

  boton.append(texto);

  boton.addEventListener("click", () => {
    grupo.estado = grupo.estado === "ON" ? "OFF" : "ON";

    alModificar();

    reconstruirTabla();
  });

  return boton;
}
