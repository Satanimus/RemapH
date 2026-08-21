// ======================================================
// ui_Separador
// ------------------------------------------------------
// Header de un Separador: div.fila-separador con Estado,
// Opciones ("⁝") y Nota — sin las celdas App/Trigger/Tipo/
// Acción/Extra que sí tiene una fila normal. No lleva celda
// de número: el botón expandir/contraer vive en el carril
// (ver comp_separador_expandir.ts).
// ======================================================

import type { SeparadorPerfil } from "../core/core_perfil";

import { crearEstadoSeparador } from "../componentes/comp_separador_estado";
import { crearBoton } from "../componentes/comp_boton";
import { crearNota } from "../componentes/comp_controles";
import { abrirPopupSeparadores } from "../componentes/comp_popup_separadores";
import { COLUMNAS } from "./ui_columnas";

// [FIX] Ancho tomado de la misma fuente única de verdad que usa
// ui_fila.ts para las filas normales — antes estas dos celdas no
// llevaban ningún ancho inline y quedaban del tamaño de su
// contenido, desalineadas con las columnas Estado/Opciones de la
// cabecera y de una fila normal (y sin reaccionar al redimensionado
// de columnas, ver ui_redimension_columnas.ts).
const anchoEstado = COLUMNAS.find((col) => col.id === "estado")?.ancho ?? "";
const anchoOpciones =
  COLUMNAS.find((col) => col.id === "opciones")?.ancho ?? "";

export function crearSeparadorHeader(
  separador: SeparadorPerfil,
  alModificar: () => void,
): HTMLElement {
  const fila = document.createElement("div");

  fila.className = "fila-separador";

  fila.dataset.id = separador.id;

  if (separador.color) {
    fila.style.setProperty("--separador-color", `var(--tag-${separador.color})`);
  } else {
    fila.style.removeProperty("--separador-color");
  }

  const celdaEstado = document.createElement("div");

  celdaEstado.className = "celda separador-estado";

  celdaEstado.style.width = anchoEstado;
  celdaEstado.style.flexBasis = anchoEstado;

  celdaEstado.append(crearEstadoSeparador(separador, alModificar));

  const celdaOpciones = document.createElement("div");

  celdaOpciones.className = "celda separador-opciones";

  celdaOpciones.style.width = anchoOpciones;
  celdaOpciones.style.flexBasis = anchoOpciones;

  const botonOpciones = crearBoton({
    texto: "⁝",
    titulo: "Opciones de separador",
    clase: "opciones-asa",
  });

  botonOpciones.addEventListener("click", (evento) => {
    abrirPopupSeparadores(evento, separador, alModificar);
  });

  celdaOpciones.append(botonOpciones);

  const celdaNota = document.createElement("div");

  celdaNota.className = "celda nota-separador";

  celdaNota.append(crearNota(separador));

  fila.append(celdaEstado, celdaOpciones, celdaNota);

  return fila;
}
