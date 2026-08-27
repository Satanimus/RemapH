// ======================================================
// ui_Fila
//
// ======================================================

import { crearContextoFila } from "../core/core_contexto_fila";

import { COLUMNAS } from "./ui_columnas";

import { crearCapturador } from "../componentes/comp_capturador";

import { crearOpciones } from "../componentes/comp_opciones";

import type { FilaPerfil } from "../core/core_perfil";

import {
  crearTipo,
  crearExtra,
  crearApp,
  crearNota,
} from "../componentes/comp_controles";

import { crearAccion } from "../componentes/comp_accion";

// ======================================================
// CREAR FILA
// ======================================================

export function crearFila(
  filaPerfil: FilaPerfil,
  esUltima: boolean,
  alModificar: () => void = () => {},
  infoSeparador?: { color: string; primera: boolean; ultima: boolean },
): HTMLElement {
  const contexto = crearContextoFila(filaPerfil.id);

  const fila = document.createElement("div");

  fila.className = "fila";

  fila.dataset.id = contexto.id;

  if (filaPerfil.color) {
    fila.style.setProperty("--fila-color", `var(--tag-${filaPerfil.color})`);
  } else {
    fila.style.removeProperty("--fila-color");
  }

  COLUMNAS.forEach((col) => {
    const celda = document.createElement("div");

    celda.className = `celda grupo-${col.grupo}`;

    celda.dataset.columna = col.id;

    celda.style.width = col.ancho;

    celda.style.flexBasis = col.ancho;

    // El tinte de separador (fondo + bordes) cubre toda la fila,
    // incluida Nota, para que el marco quede parejo de punta a
    // punta en vez de sentirse cargado a la izquierda.
    if (infoSeparador) {
      celda.style.setProperty(
        "--separador-color",
        infoSeparador.color
          ? `var(--tag-${infoSeparador.color})`
          : "var(--border-light)",
      );

      celda.classList.add("en-separador");

      if (col.id === "opciones") {
        celda.classList.add("en-separador-inicio");
      }

      // "en-separador-fin" da la esquina redondeada del lado
      // derecho y el borde de acento (color real, o border-light
      // como fallback si no tiene color — igual que en-separador-
      // inicio del otro lado). Ahora cae en Nota, la última
      // columna de la fila.
      if (col.id === "nota") {
        celda.classList.add("en-separador-fin");
      }

      if (infoSeparador.primera) {
        celda.classList.add("primera-del-separador");
      }

      if (infoSeparador.ultima) {
        celda.classList.add("ultima-del-separador");
      }
    }

    switch (col.id) {
      case "opciones":
        celda.append(
          crearOpciones(contexto, filaPerfil, esUltima, alModificar),
        );

        break;

      case "app":
        celda.append(crearApp(contexto, filaPerfil, alModificar));

        break;

      case "trigger":
        celda.append(
          crearCapturador(contexto, filaPerfil, "Trigger", alModificar),
        );

        break;

      case "tipo":
        celda.append(crearTipo(contexto, filaPerfil, alModificar));

        break;

      case "accion":
        celda.dataset.control = "accion";

        celda.append(crearAccion(contexto, filaPerfil, alModificar));

        break;

      case "extra":
        celda.dataset.control = "extra";

        celda.append(crearExtra(contexto, filaPerfil, alModificar));

        break;

      case "nota":
        celda.append(crearNota(filaPerfil, alModificar));

        break;
    }

    fila.append(celda);
  });

  return fila;
}
