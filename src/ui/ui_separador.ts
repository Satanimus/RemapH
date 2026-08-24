// ======================================================
// ui_Separador
// ------------------------------------------------------
// Header de un Separador: div.fila-separador con Opciones
// ("⁝") y Nota — sin las celdas App/Trigger/Tipo/Acción/
// Extra que sí tiene una fila normal. No lleva celda de
// número ni de estado: el botón expandir/contraer y el
// On/Off viven en el carril, lado a lado (ver
// comp_separador_expandir.ts).
// ======================================================

import type { SeparadorPerfil } from "../core/core_perfil";

import { crearBoton } from "../componentes/comp_boton";
import { crearNota } from "../componentes/comp_controles";
import { abrirPopupSeparadores } from "../componentes/comp_popup_separadores";
import { COLUMNAS } from "./ui_columnas";

// [FIX] Ancho de Opciones tomado de la misma fuente única de verdad
// que usa ui_fila.ts para las filas normales — antes esta celda no
// llevaba ningún ancho inline y quedaba del tamaño de su contenido,
// desalineada con la columna Opciones de la cabecera y de una fila
// normal (y sin reaccionar al redimensionado de columnas, ver
// ui_redimension_columnas.ts).
const anchoOpciones =
  COLUMNAS.find((col) => col.id === "opciones")?.ancho ?? "";

export function crearSeparadorHeader(
  separador: SeparadorPerfil,
  alModificar: () => void,
): HTMLElement {
  const fila = document.createElement("div");

  fila.className = "fila-separador";

  fila.dataset.id = separador.id;

  // Etapa E6: el tinte de separador (fondo/border-radius de
  // .fila-separador) debe terminar en el borde derecho de "Extra",
  // igual que en fila normal — se mueve --separador-color y el
  // fondo del contenedor "fila" a un wrapper interno (celdaCuerpo)
  // que sí mide Opciones→Extra. celdaNota queda fuera de ese
  // wrapper, sin heredar la variable, con estilo de nota al margen.
  const celdaCuerpo = document.createElement("div");

  celdaCuerpo.className = "separador-cuerpo";

  if (separador.color) {
    celdaCuerpo.style.setProperty(
      "--separador-color",
      `var(--tag-${separador.color})`,
    );
  } else {
    celdaCuerpo.style.removeProperty("--separador-color");
  }

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

  celdaCuerpo.append(celdaOpciones);

  const celdaNota = document.createElement("div");

  celdaNota.className = "celda nota-separador";

  celdaNota.append(crearNota(separador, alModificar));

  fila.append(celdaCuerpo, celdaNota);

  return fila;
}
