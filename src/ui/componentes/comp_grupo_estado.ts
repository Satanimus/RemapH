// ======================================================
// 🟢🔴 comp_Grupo_Estado
// ------------------------------------------------------
// Botón Estado del header de un Separador.
// Al hacer clic: fuerza ese estado en todas las filas del
// tramo (cascada descendente, Regla 14). Muestra indicador
// gris cuando estadoVisual del separador es "mixto".
// ======================================================

import type { SeparadorPerfil } from "../../core/core_perfil";
import {
  aplicarCascadaDescendente,
  recomputarCascadaAscendente,
} from "../../core/core_agrupacion";
import { obtenerPerfilUi } from "../../core/core_perfil_ui";
import { reconstruirTabla } from "../ui_tabla_control";

export function crearEstadoGrupo(
  separador: SeparadorPerfil,
  alModificar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn estado-toggle";

  boton.dataset.estado = separador.estado === "ON" ? "on" : "off";

  if (separador.estadoVisual === "mixto") {
    boton.classList.add("estado-grupo-mixto");
  }

  const texto = document.createElement("span");

  texto.textContent = separador.estado;

  boton.append(texto);

  boton.addEventListener("click", () => {
    separador.estado = separador.estado === "ON" ? "OFF" : "ON";

    const perfilActual = obtenerPerfilUi();

    const indice = perfilActual.filas.indexOf(separador);

    if (indice !== -1) {
      aplicarCascadaDescendente(perfilActual.filas, indice, separador.estado);
    }

    recomputarCascadaAscendente(perfilActual.filas);

    alModificar();

    reconstruirTabla();
  });

  return boton;
}
