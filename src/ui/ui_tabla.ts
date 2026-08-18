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
  registrarActivarModoMover,
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
  // 🔢 CARRIL DE NÚMEROS (Etapa 9B)
  // ------------------------------------------------------
  // Fuera de .viewport a propósito: la numeración no es una
  // columna más de la fila, es un indicador de fondo fuera
  // de la tabla que arrastra/soltar (spec: "no pertenece a
  // la fila, no se mueve con ella"). Un espaciador imita la
  // altura de la cabecera (que sí scrollea dentro de
  // .viewport, no tiene position:sticky) para que el número
  // 1 quede a la altura de la primera fila. La sincronía de
  // scroll es un simple espejo de scrollTop — el contenido
  // total (espaciador + N filas de --row-height) mide lo
  // mismo en los dos lados porque usan las mismas variables
  // de alto que .cabecera/.fila.
  // ==================================================

  const carrilNumeros = document.createElement("div");

  carrilNumeros.className = "carril-numeros";

  const carrilEspaciador = document.createElement("div");

  carrilEspaciador.className = "carril-numeros-espaciador";

  const carrilLista = document.createElement("div");

  carrilLista.className = "carril-numeros-lista";

  carrilNumeros.append(carrilEspaciador, carrilLista);

  const reconstruirCarrilNumeros = (total: number): void => {
    carrilLista.replaceChildren();

    for (let i = 1; i <= total; i++) {
      const item = document.createElement("div");

      item.className = "carril-numero";
      item.textContent = String(i);

      carrilLista.append(item);
    }
  };

  viewport.addEventListener("scroll", () => {
    carrilNumeros.scrollTop = viewport.scrollTop;
  });

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

  registrarActivarModoMover((id) =>
    controladorArrastre.activarModoMoverPara(id),
  );

  // Registra una fila ya insertada en el DOM (asa = botón
  // "⁝", ver comp_opciones.ts::crearOpciones).
  const registrarFilaArrastrable = (filaElemento: HTMLElement): void => {
    const id = filaElemento.dataset.id;

    if (!id) {
      return;
    }

    const asa = filaElemento.querySelector<HTMLElement>(".opciones-asa");

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
        indice === perfil.filas.length - 1,
        alModificar,
      );

      filas.append(filaElemento);

      registrarFilaArrastrable(filaElemento);
    });

    reconstruirCarrilNumeros(perfil.filas.length);
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
      indice === perfil.filas.length - 1,
      alModificar,
    );

    filaActual.replaceWith(filaNueva);

    registrarFilaArrastrable(filaNueva);
  };

  // ==================================================
  // ✏️ CAMBIO VISUAL EN FILA
  // ------------------------------------------------------
  // Un clic sobre un control de cualquier columna que NO
  // sea el asa (Opciones) tiene prioridad y saca del modo
  // Mover (spec Etapa 9, punto 3) — el propio asa maneja
  // su clic corto/mantenido por separado (ver
  // comp_opciones.ts + util_arrastrable.ts).
  // ==================================================

  filas.addEventListener("click", (evento) => {
    const objetivo = evento.target as HTMLElement;

    const control = objetivo.closest("button, select, input");

    if (!control) {
      return;
    }

    const celda = objetivo.closest<HTMLElement>(".celda");

    if (celda?.dataset.columna !== "opciones") {
      controladorArrastre.salirModoMover();
    }

    alModificar();
  });

  viewport.append(cabecera, filas);

  tabla.append(carrilNumeros, viewport);

  registrarReconstruccion(reconstruirTabla, reconstruirFila);

  return tabla;
}
