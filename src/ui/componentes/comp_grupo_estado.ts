// ======================================================
// 🟢🔴 comp_Grupo_Estado
// ------------------------------------------------------
// Botón Estado del header de una Agrupación.
// Al hacer clic: fuerza ese estado en todas las filas
// contenidas (Etapa E). Muestra indicador gris cuando
// alguna fila ya no coincide con el estado del grupo.
// ======================================================

import type { AgrupacionPerfil } from "../../core/core_perfil";
import {
  calcularPertenencia,
  estadoGrupoVigente,
} from "../../core/core_agrupacion";
import { obtenerPerfilUi } from "../../core/core_perfil_ui";
import { reconstruirTabla } from "../ui_tabla_control";

export function crearEstadoGrupo(
  grupo: AgrupacionPerfil,
  alModificar: () => void,
): HTMLButtonElement {
  const perfil = obtenerPerfilUi();

  const { rangoPorGrupo } = calcularPertenencia(
    perfil.grupos,
    perfil.filas.length,
  );

  const rango = rangoPorGrupo.get(grupo.id) ?? { inicio: 0, fin: 0 };

  const vigente = estadoGrupoVigente(grupo, perfil.filas, rango);

  const boton = document.createElement("button");

  boton.className = "ui-btn estado-toggle";

  boton.dataset.estado = grupo.estado === "ON" ? "on" : "off";

  if (vigente === "mixto") {
    boton.classList.add("estado-grupo-mixto");
  }

  const texto = document.createElement("span");

  texto.textContent = grupo.estado;

  boton.append(texto);

  boton.addEventListener("click", () => {
    grupo.estado = grupo.estado === "ON" ? "OFF" : "ON";

    const perfilActual = obtenerPerfilUi();

    const { rangoPorGrupo: rangos } = calcularPertenencia(
      perfilActual.grupos,
      perfilActual.filas.length,
    );

    const rangoActual = rangos.get(grupo.id);

    if (rangoActual) {
      for (let i = rangoActual.inicio; i < rangoActual.fin; i++) {
        perfilActual.filas[i].estado = grupo.estado;
      }
    }

    alModificar();

    reconstruirTabla();
  });

  return boton;
}
