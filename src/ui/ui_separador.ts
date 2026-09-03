// ======================================================
// ui_Separador
// ------------------------------------------------------
// Header de un Separador: div.fila-separador con Opciones
// ("⁝") y Nota — sin las celdas App/Trigger/Tipo/Acción/
// Extra que sí tiene una fila normal. No lleva celda de
// número ni de estado: el botón expandir/contraer y el
// On/Off viven en el carril, lado a lado (ver
// comp_separador_expandir.ts).
//
// Opciones usa el mismo botón "⁝" toggle y los mismos 3
// botones (crearBotonesOpcionesExtra) que la fila normal
// (ver comp_opciones.ts). Eliminar un separador solo elimina
// el separador (mismo criterio/alcance que eliminarFilaPorId
// para una fila): las filas que contenía quedan donde están,
// sin doble confirmación (los separadores no tienen "acción"
// configurable que proteger).
//
// [FIX] Nota funciona como Nombre del separador: ocupa
// desde donde arrancaría la columna App hasta el borde
// derecho de la fila (antes quedaba un tramo vacío
// coloreado entre Opciones y Extra, y la Nota arrancaba
// recién después de Extra). El color/fondo del separador
// ahora se pinta en la fila completa (fila-separador), no
// en un wrapper intermedio — así el botón Opciones queda
// en el mismo ancho/posición que la celda Opciones de una
// fila normal, sin una celda "cuerpo" de por medio que
// antes desalineaba el resto del contenido.
// ======================================================

import type { SeparadorPerfil } from "../core/core_perfil";

import { crearBoton } from "../componentes/comp_boton";
import { crearNota } from "../componentes/comp_controles";
import { crearBotonesOpcionesExtra } from "../componentes/comp_opciones";
import { abrirPopupColorSeparador } from "../componentes/comp_popup_separadores";
import {
  clonarSeparadoresPorId,
  eliminarFilaPorId,
} from "../core/core_perfil_acciones";
import {
  reconstruirTabla,
  opcionesColumnaEstaExpandida,
  alternarOpcionesColumna,
} from "./ui_tabla_control";
import { COLUMNAS } from "./ui_columnas";

// Ancho de Opciones tomado de la misma fuente única de verdad
// que usa ui_fila.ts para las filas normales.
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
    fila.style.setProperty(
      "--separador-color",
      `var(--tag-${separador.color})`,
    );
  } else {
    fila.style.removeProperty("--separador-color");
  }

  const celdaOpciones = document.createElement("div");

  celdaOpciones.className = "celda separador-opciones";

  // [FIX bug 3] Sin este atributo, la regla CSS que da min-width:150px
  // cuando hay .opciones-extra (styl_tabla.css, selector
  // .celda[data-columna="opciones"]:has(.opciones-extra)) nunca
  // matcheaba acá — la celda quedaba fija en su ancho de columna y
  // el límite visual entre Opciones y Nota no se desplazaba al
  // expandir, a diferencia de la fila normal (ver ui_fila.ts).
  celdaOpciones.dataset.columna = "opciones";

  celdaOpciones.style.width = anchoOpciones;
  celdaOpciones.style.flexBasis = anchoOpciones;

  const contenedorOpciones = document.createElement("div");

  contenedorOpciones.className = "opciones-celda";

  const botonOpciones = crearBoton({
    texto: "⁝",
    titulo: "Opciones de separador",
    clase: "opciones-asa",
  });

  botonOpciones.addEventListener("click", () => {
    alternarOpcionesColumna();
  });

  contenedorOpciones.append(botonOpciones);

  if (opcionesColumnaEstaExpandida()) {
    contenedorOpciones.append(
      crearBotonesOpcionesExtra({
        onAbrirColor: (evento) => {
          abrirPopupColorSeparador(evento, separador);
        },
        onDuplicar: () => {
          clonarSeparadoresPorId(separador.id);
          alModificar();
          reconstruirTabla();
        },
        onEliminar: () => {
          eliminarFilaPorId(separador.id);
          alModificar();
          reconstruirTabla();
        },
        requiereConfirmacion: false,
      }),
    );
  }

  celdaOpciones.append(contenedorOpciones);

  const celdaNota = document.createElement("div");

  celdaNota.className = "celda nota-separador";

  celdaNota.append(crearNota(separador, alModificar));

  fila.append(celdaOpciones, celdaNota);

  return fila;
}
