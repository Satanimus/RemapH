// ======================================================
// ⁝ comp_Opciones
// ------------------------------------------------------
// Columna "Opciones": botón "⁝" con popup (Color / Clonar /
// Mover / Eliminar). Reemplaza a comp_numero.ts (Etapa C del
// plan "tabla-perfil-ui") — el número ordinal sigue viviendo
// en el carril fijo a la izquierda de la tabla (ver
// ui_tabla.ts), sincronizado con el scroll pero indiferente
// a esta fila. Este botón sirve también de asa para el
// componente de arrastre (util_arrastrable.ts): clic corto
// abre este popup, clic mantenido lo maneja el componente
// por su cuenta (misma clase "opciones-asa" usada para
// ubicar el botón vía querySelector tras cada render — ver
// ui_tabla.ts, Etapa D).
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";
import type { FilaPerfil } from "../../core/core_perfil";
import { crearFila } from "../../core/core_perfil";
import { crearBoton } from "./comp_boton";
import { abrirPopupOpciones } from "./comp_popup_abrir";
import { obtenerPerfilUi } from "../../core/core_perfil_ui";
import { reconstruirTabla } from "../ui_tabla_control";

export function crearOpciones(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  esUltima: boolean,
  alModificar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  contenedor.className = "opciones-celda";

  const boton = crearBoton({
    texto: "⁝",
    titulo: "Opciones de fila",
    clase: "opciones-asa",
  });

  boton.addEventListener("click", (evento) => {
    abrirPopupOpciones(evento, contexto, filaPerfil, alModificar);
  });

  contenedor.append(boton);

  // ==================================================
  // ➕ AGREGAR FILA (solo debajo de la última fila)
  // ==================================================

  if (esUltima) {
    const botonAgregar = document.createElement("button");

    botonAgregar.className = "btn-agregar-fila";
    botonAgregar.type = "button";
    botonAgregar.title = "Agregar fila";

    const simbolo = document.createElement("span");

    simbolo.textContent = "+";

    botonAgregar.append(simbolo);

    botonAgregar.addEventListener("click", (evento) => {
      evento.stopPropagation();

      const perfil = obtenerPerfilUi();

      perfil.filas.push(crearFila());

      alModificar();
      reconstruirTabla();
    });

    contenedor.append(botonAgregar);
  }

  return contenedor;
}
