// ======================================================
// ui_Fila
//
// ======================================================

import { crearContextoFila } from "../core/core_contexto_fila";

import { COLUMNAS } from "./ui_columnas";

import { crearCapturador } from "./componentes/comp_capturador";

import { crearOpciones } from "./componentes/comp_opciones";

import type { FilaPerfil } from "../core/core_perfil";

import {
  crearEstado,
  crearTipo,
  crearExtra,
  crearApp,
  crearNota,
} from "./componentes/comp_controles";

import { crearAccion } from "./componentes/comp_accion";

// ======================================================
// CREAR FILA
// ======================================================

export function crearFila(
  filaPerfil: FilaPerfil,
  esUltima: boolean,
  alModificar: () => void = () => {},
  infoGrupo?: { color: string; primera: boolean; ultima: boolean },
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

  if (infoGrupo) {
    fila.style.setProperty(
      "--grupo-color",
      infoGrupo.color ? `var(--tag-${infoGrupo.color})` : "var(--border-light)",
    );

    fila.classList.add("en-grupo");

    if (infoGrupo.primera) {
      fila.classList.add("primera-del-grupo");
    }

    if (infoGrupo.ultima) {
      fila.classList.add("ultima-del-grupo");
    }
  } else {
    fila.style.removeProperty("--grupo-color");
  }

  COLUMNAS.forEach((col) => {
    const celda = document.createElement("div");

    celda.className = `celda grupo-${col.grupo}`;

    celda.dataset.columna = col.id;

    celda.style.width = col.ancho;

    celda.style.flexBasis = col.ancho;

    switch (col.id) {
      case "estado":
        celda.append(crearEstado(contexto, filaPerfil));

        break;

      case "opciones":
        celda.append(
          crearOpciones(contexto, filaPerfil, esUltima, alModificar),
        );

        break;

      case "app":
        celda.append(crearApp(contexto, filaPerfil));

        break;

      case "trigger":
        celda.append(
          crearCapturador(contexto, filaPerfil, "Trigger", alModificar),
        );

        break;

      case "tipo":
        celda.append(crearTipo(contexto, filaPerfil));

        break;

      case "accion":
        celda.dataset.control = "accion";

        celda.append(crearAccion(contexto, filaPerfil, alModificar));

        break;

      case "extra":
        celda.dataset.control = "extra";

        celda.append(crearExtra(contexto, filaPerfil));

        break;

      case "nota":
        celda.append(crearNota(filaPerfil));

        break;
    }

    fila.append(celda);
  });

  return fila;
}
