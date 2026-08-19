// ======================================================
// 🟢🔴 comp_Grupo_Estado
// ------------------------------------------------------
// Botón Estado del header de un Separador.
// Al hacer clic: fuerza ese estado en todas las filas del
// tramo (cascada descendente, Regla 14). Muestra indicador
// gris cuando estadoVisual del separador es "mixto".
// ======================================================

import type { FilaPerfil, SeparadorPerfil } from "../../core/core_perfil";
import {
  aplicarCascadaDescendente,
  esSeparador,
  obtenerTramoDeSeparador,
  recomputarCascadaAscendente,
} from "../../core/core_agrupacion";
import { obtenerPerfilUi } from "../../core/core_perfil_ui";
import { reconstruirTabla } from "../ui_tabla_control";
import { filaTieneConflicto } from "../../core/core_conflictos";
import { filaTieneAdvertencia } from "../../core/core_advertencias_compilacion";

// Tercer estado (además de on/off/mixto): si alguna fila del tramo
// propio de este separador (el más cercano por arriba) está "en
// alerta" (conflicto entre filas o advertencia de la última
// compilación — mismo criterio que crearEstado en comp_controles.ts),
// el botón se muestra solo como ícono rojo, sin texto ON/OFF.
function tramoTieneAlerta(separador: SeparadorPerfil): boolean {
  const perfilActual = obtenerPerfilUi();

  const indice = perfilActual.filas.indexOf(separador);

  if (indice === -1) {
    return false;
  }

  const tramo = obtenerTramoDeSeparador(perfilActual.filas, indice);

  const filasNormales = perfilActual.filas.filter(
    (item): item is FilaPerfil => !esSeparador(item),
  );

  return tramo.some(
    (fila) =>
      filaTieneConflicto(fila.id, filasNormales) ||
      filaTieneAdvertencia(fila.id, filasNormales),
  );
}

export function crearEstadoGrupo(
  separador: SeparadorPerfil,
  alModificar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn estado-toggle";

  const hayAlerta = tramoTieneAlerta(separador);
  const esMixto = !hayAlerta && separador.estadoVisual === "mixto";

  boton.dataset.estado = separador.estado === "ON" ? "on" : "off";
  boton.dataset.conflicto = hayAlerta ? "true" : "false";

  if (hayAlerta) {
    const alerta = document.createElement("span");

    alerta.className = "estado-alerta";

    alerta.textContent = "⚠";

    boton.append(alerta);
  } else if (esMixto) {
    boton.classList.add("estado-grupo-mixto");

    const texto = document.createElement("span");

    texto.textContent = "⨂";

    boton.append(texto);
  } else {
    const texto = document.createElement("span");

    texto.textContent = separador.estado === "ON" ? "◉" : "⨉";

    boton.append(texto);
  }

  boton.addEventListener("click", (evento) => {
    if (hayAlerta) {
      evento.stopPropagation();

      return;
    }

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
