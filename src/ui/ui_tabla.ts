// ======================================================
// ui_Tabla
//
// ======================================================

import { COLUMNAS } from "./ui_columnas";

import { crearFila } from "./ui_fila";

import { obtenerPerfilUi } from "../core/core_perfil_ui";

import type { FilaPerfil } from "../core/core_perfil";

import {
  registrarReconstruccion,
  registrarSalirModoMover,
} from "./ui_tabla_control";

import { activarRedimensionColumnas } from "./ui_redimension_columnas";

import { crearControladorArrastre } from "./util/util_arrastrable";

// ======================================================
// CREAR TABLA
// ======================================================

export function crearTabla(alModificar: () => void): HTMLElement {
  const tabla = document.createElement("section");

  tabla.className = "tabla";

  const viewport = document.createElement("div");

  viewport.className = "viewport";

  const cabecera = document.createElement("div");

  cabecera.className = "cabecera";

  COLUMNAS.forEach((col) => {
    const celda = document.createElement("div");

    celda.className = `cabecera-celda grupo-${col.grupo}`;

    celda.dataset.columna = col.id;

    celda.style.width = col.ancho;

    celda.style.flexBasis = col.ancho;

    celda.textContent = col.titulo;

    const divisor = document.createElement("div");

    divisor.className = "divisor-columna";

    celda.append(divisor);

    divisor.style.pointerEvents = "auto";

    cabecera.append(celda);
  });

  activarRedimensionColumnas(cabecera);

  const filas = document.createElement("div");

  filas.className = "filas";

  // ==================================================
  // ⁝⁝ ARRASTRAR Y SOLTAR (util_arrastrable.ts, Etapa 9)
  // ------------------------------------------------------
  // El controlador vive una sola vez para toda la vida de
  // la tabla (a diferencia del editor de Macro, que la crea
  // y destruye por cada apertura de popup) — no hace falta
  // llamar destruir(). onReordenar solo sincroniza el
  // array del perfil: el reordenamiento visual del DOM ya
  // lo hizo el componente antes de llamarlo.
  // ==================================================

  const controladorArrastre = crearControladorArrastre({
    contenedor: filas,

    obtenerOrdenIds: () => obtenerPerfilUi().filas.map((fila) => fila.id),

    onReordenar: (nuevoOrden) => {
      const perfil = obtenerPerfilUi();

      const porId = new Map(perfil.filas.map((fila) => [fila.id, fila]));

      perfil.filas = nuevoOrden
        .map((id) => porId.get(id))
        .filter((fila): fila is FilaPerfil => !!fila);

      alModificar();
    },
  });

  registrarSalirModoMover(() => controladorArrastre.salirModoMover());

  // Registra una fila ya insertada en el DOM (asa = botón
  // "N ▾", ver comp_numero.ts::crearNumero).
  const registrarFilaArrastrable = (filaElemento: HTMLElement): void => {
    const id = filaElemento.dataset.id;

    if (!id) {
      return;
    }

    const asa = filaElemento.querySelector<HTMLElement>(".numero-asa");

    if (asa) {
      controladorArrastre.registrarFila(id, filaElemento, asa);
    }
  };

  // ==================================================
  // RECONSTRUIR TABLA
  // ==================================================

  const reconstruirTabla = (): void => {
    filas.replaceChildren();

    const perfil = obtenerPerfilUi();

    perfil.filas.forEach((fila, indice) => {
      const filaElemento = crearFila(
        fila,
        indice + 1,
        perfil.filas.length,
        alModificar,
      );

      filas.append(filaElemento);

      registrarFilaArrastrable(filaElemento);
    });
  };

  reconstruirTabla();

  // ==================================================
  // RECONSTRUIR FILA
  // ==================================================

  const reconstruirFila = (id: string): void => {
    const perfil = obtenerPerfilUi();

    const indice = perfil.filas.findIndex((fila) => fila.id === id);

    if (indice < 0) {
      return;
    }

    const filaActual = filas.querySelector(`[data-id="${id}"]`);

    if (!filaActual) {
      return;
    }

    const filaNueva = crearFila(
      perfil.filas[indice],
      indice + 1,
      perfil.filas.length,
      alModificar,
    );

    filaActual.replaceWith(filaNueva);

    registrarFilaArrastrable(filaNueva);
  };

  // ==================================================
  // ✏️ CAMBIO VISUAL EN FILA
  // ------------------------------------------------------
  // Un clic sobre un control de cualquier columna que NO
  // sea el asa (número) tiene prioridad y saca del modo
  // Mover (spec Etapa 9, punto 3) — el propio asa maneja
  // su clic corto/mantenido por separado (ver comp_numero.ts
  // + util_arrastrable.ts).
  // ==================================================

  filas.addEventListener("click", (evento) => {
    const objetivo = evento.target as HTMLElement;

    const control = objetivo.closest("button, select, input");

    if (!control) {
      return;
    }

    const celda = objetivo.closest<HTMLElement>(".celda");

    if (celda?.dataset.columna !== "numero") {
      controladorArrastre.salirModoMover();
    }

    alModificar();
  });

  viewport.append(cabecera, filas);

  tabla.append(viewport);

  registrarReconstruccion(reconstruirTabla, reconstruirFila);

  return tabla;
}
