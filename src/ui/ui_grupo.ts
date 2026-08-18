// ======================================================
// ui_Grupo
// ------------------------------------------------------
// Header de una Agrupación: div.fila-grupo con Estado,
// Opciones ("⁝") y Nota — sin las celdas App/Trigger/Tipo/
// Acción/Extra que sí tiene una fila normal. No lleva celda
// de número: el botón expandir/contraer vive en el carril
// (ver comp_grupo_expandir.ts).
// ======================================================

import type { AgrupacionPerfil } from "../core/core_perfil";

import { crearEstadoGrupo } from "./componentes/comp_grupo_estado";
import { crearBoton } from "./componentes/comp_boton";
import { crearNota } from "./componentes/comp_controles";
import { abrirPopupAgrupacion } from "./componentes/comp_popup_agrupacion";

export function crearGrupoHeader(
  grupo: AgrupacionPerfil,
  alModificar: () => void,
): HTMLElement {
  const fila = document.createElement("div");

  fila.className = "fila-grupo";

  fila.dataset.id = grupo.id;

  if (grupo.color) {
    fila.style.setProperty("--grupo-color", `var(--tag-${grupo.color})`);
  } else {
    fila.style.removeProperty("--grupo-color");
  }

  const celdaEstado = document.createElement("div");

  celdaEstado.className = "celda grupo-estado";

  celdaEstado.append(crearEstadoGrupo(grupo, alModificar));

  const celdaOpciones = document.createElement("div");

  celdaOpciones.className = "celda grupo-opciones";

  const botonOpciones = crearBoton({
    texto: "⁝",
    titulo: "Opciones de grupo",
    clase: "opciones-asa",
  });

  botonOpciones.addEventListener("click", (evento) => {
    abrirPopupAgrupacion(evento, grupo, alModificar);
  });

  celdaOpciones.append(botonOpciones);

  const celdaNota = document.createElement("div");

  celdaNota.className = "celda nota-grupo";

  celdaNota.append(crearNota(grupo));

  fila.append(celdaEstado, celdaOpciones, celdaNota);

  return fila;
}
