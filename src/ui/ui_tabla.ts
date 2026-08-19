// ======================================================
// ui_Tabla
//
// ======================================================

import { COLUMNAS } from "./ui_columnas";

import { crearFila } from "./ui_fila";

import { crearSeparadorHeader } from "./ui_grupo";

import { crearExpandirGrupo } from "./componentes/comp_grupo_expandir";

import { obtenerPerfilUi } from "../core/core_perfil_ui";

import { construirPlanVisual, esSeparador } from "../core/core_agrupacion";

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

  viewport.addEventListener("scroll", () => {
    carrilNumeros.scrollTop = viewport.scrollTop;
  });

  // ==================================================
  // ⁝⁝ ARRASTRAR Y SOLTAR (util_arrastrable.ts, Etapa 9)
  // ------------------------------------------------------
  // El controlador vive una sola vez para toda la vida de
  // la tabla (a diferencia del editor de Macro, que la crea
  // y destruye por cada apertura de popup) — no hace falta
  // llamar destruir(). onReordenar sincroniza el array del
  // perfil y ADEMÁS reconstruye la tabla entera: el
  // reordenamiento visual que ya hizo el componente es solo
  // un swap de posición de los nodos existentes, no vuelve a
  // calcular pertenencia a grupo — sin este reconstruirTabla()
  // el carril de números/expandir y los colores/bordes de
  // grupo de cada fila quedan con los valores de ANTES del
  // movimiento (ver bugs de sincronía y de color de fondo).
  // ==================================================

  const controladorArrastre = crearControladorArrastre({
    contenedor: filas,

    obtenerOrdenIds: () => {
      const plan = construirPlanVisual(obtenerPerfilUi());

      return plan.map((item) =>
        item.tipo === "separador" ? item.separador.id : item.fila.id,
      );
    },

    onReordenar: (nuevoOrden) => {
      const perfil = obtenerPerfilUi();

      // Regla 9/10/12: el nuevo orden mezcla ids de fila y de
      // separador tal cual el usuario los arrastró — es
      // directamente el nuevo array de filas del perfil, sin
      // rangos ni recálculo de pertenencia (eso lo deriva
      // construirPlanVisual en cada render).
      const porId = new Map(perfil.filas.map((item) => [item.id, item]));

      perfil.filas = nuevoOrden
        .map((id) => porId.get(id))
        .filter((item): item is (typeof perfil.filas)[number] => !!item);

      alModificar();

      reconstruirTabla();
    },

    obtenerIdsSeparadores: () =>
      obtenerPerfilUi()
        .filas.filter(esSeparador)
        .map((separador) => separador.id),

    // Regla 11: si se arrastra un separador contraído, se expande
    // automáticamente al iniciar el gesto de arrastre — mutamos el
    // modelo y reconstruimos la tabla acá mismo (mismo criterio que
    // el botón ↴/≫ de crearExpandirGrupo) para que las filas que
    // quedaban ocultas por el colapso ya estén en el DOM cuando
    // iniciarArrastre calcule posiciones.
    esSeparadorContraido: (id) => {
      const item = obtenerPerfilUi().filas.find((f) => f.id === id);

      if (!item || !esSeparador(item) || item.expandido) {
        return false;
      }

      item.expandido = true;

      alModificar();

      reconstruirTabla();

      return true;
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

    carrilLista.replaceChildren();

    const perfil = obtenerPerfilUi();

    const plan = construirPlanVisual(perfil);

    // Regla 18: la numeración del carril solo cuenta filas normales,
    // saltando los separadores en la secuencia.
    let numeroVisible = 0;

    // Último índice de plan que es una fila, para saber cuál es la
    // "última" fila visible cuando no pertenece a ningún separador
    // (zona superior, Regla 4) — item.separador.ultima ya resuelve
    // ese caso cuando sí hay separador activo.
    let ultimoIndiceFilaEnPlan = -1;

    plan.forEach((item, indicePlan) => {
      if (item.tipo === "fila") {
        ultimoIndiceFilaEnPlan = indicePlan;
      }
    });

    plan.forEach((item, indicePlan) => {
      if (item.tipo === "fila") {
        const esUltima = item.separador
          ? item.separador.ultima
          : indicePlan === ultimoIndiceFilaEnPlan;

        const filaElemento = crearFila(
          item.fila,
          esUltima,
          alModificar,
          item.separador,
        );

        filas.append(filaElemento);

        registrarFilaArrastrable(filaElemento);

        numeroVisible += 1;

        const numero = document.createElement("div");

        numero.className = "carril-numero";
        numero.textContent = String(numeroVisible);

        carrilLista.append(numero);
      } else {
        const headerElemento = crearSeparadorHeader(
          item.separador,
          alModificar,
        );

        filas.append(headerElemento);

        carrilLista.append(crearExpandirGrupo(item.separador, alModificar));

        const botonOpciones = headerElemento.querySelector(
          ".opciones-asa",
        ) as HTMLElement | null;

        if (botonOpciones) {
          controladorArrastre.registrarFila(
            item.separador.id,
            headerElemento,
            botonOpciones,
          );
        }
      }
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

    const item = perfil.filas[indice];

    // reconstruirFila es solo para filas normales — un separador
    // se re-renderiza a través de reconstruirTabla (ver
    // crearSeparadorHeader/crearExpandirGrupo), no acá.
    if (esSeparador(item)) {
      return;
    }

    const filaActual = filas.querySelector(`[data-id="${id}"]`);

    if (!filaActual) {
      return;
    }

    const filaNueva = crearFila(
      item,
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
