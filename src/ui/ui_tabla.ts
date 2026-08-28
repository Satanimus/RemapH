// ======================================================
// ui_Tabla
//
// ======================================================

import { COLUMNAS } from "./ui_columnas";

import { crearFila } from "./ui_fila";

import { crearSeparadorHeader } from "./ui_separador";

import { crearEstadoSeparador } from "../componentes/comp_separador_estado";

import { crearExpandirSeparador } from "../componentes/comp_separador_expandir";

import { crearEstado } from "../componentes/comp_controles";

import { obtenerPerfilUi } from "../core/core_perfil_ui";

import {
  construirPlanVisual,
  construirNumerosAbsolutos,
  esSeparador,
  obtenerTramoDeSeparador,
} from "../core/core_separadores";

import type { ItemFilaPerfil, SeparadorPerfil } from "../core/core_perfil";

import {
  registrarReconstruccion,
  registrarSalirModoMover,
  registrarActivarModoMover,
} from "./ui_tabla_control";

import {
  activarRedimensionColumnas,
  ANCHOS_DEFAULT,
} from "./ui_redimension_columnas";

import { crearControladorArrastre } from "../util/util_arrastrable";

import { ATRIBUTO_AYUDA_ID } from "./ui_ayuda_hover";

// ======================================================
// CREAR TABLA
// ======================================================

export function crearTabla(alModificar: () => void): HTMLElement {
  const tabla = document.createElement("section");

  tabla.className = "tabla";

  const viewport = document.createElement("div");

  viewport.className = "viewport";

  // ==================================================
  // 🩹 DETECCIÓN DE RECORTE HORIZONTAL
  // ------------------------------------------------------
  // Bug: al angostar la ventana, el respiro fijo de --gap8 en
  // .cabecera/.fila/.fila-separador (ver styl_tabla.css) le
  // come 8px al propio box de Nota/separador en vez de quedar
  // afuera de él, apenas ese box deja de tener lugar para
  // achicarse y empieza a quedar cortado por la ventana.
  // scrollWidth > clientWidth es exactamente esa condición: el
  // contenido de la fila (Opciones..Nota) ya no entra en el
  // viewport. Solo ahí se agrega "recortado", que anula el
  // respiro (ver .viewport.recortado en styl_tabla.css) para
  // que el fondo/color siempre llegue hasta donde llega el
  // contenido real. Se reevalúa en cada resize del viewport
  // (el caso reportado: achicar la ventana) y después de cada
  // reconstrucción de la tabla (cambiar de perfil, agregar o
  // quitar filas/separadores).
  // ==================================================

  const actualizarRecorte = (): void => {
    viewport.classList.toggle(
      "recortado",
      viewport.scrollWidth > viewport.clientWidth,
    );
  };

  new ResizeObserver(actualizarRecorte).observe(viewport);

  const cabecera = document.createElement("div");

  cabecera.className = "cabecera";

  const IDS_AYUDA_COLUMNA: Record<string, string> = {
    opciones: "opciones",
    app: "app-columna",
    trigger: "trigger",
    tipo: "tipo-fila",
    accion: "accion",
    extra: "extra",
    nota: "nota-columna",
  };

  COLUMNAS.forEach((col) => {
    const celda = document.createElement("div");

    celda.className = `cabecera-celda grupo-${col.grupo}`;

    celda.dataset.columna = col.id;

    celda.style.width = col.ancho;

    celda.style.flexBasis = col.ancho;

    celda.textContent = col.titulo;

    celda.title = col.titulo;

    const idAyuda = IDS_AYUDA_COLUMNA[col.id];

    if (idAyuda) {
      celda.setAttribute(ATRIBUTO_AYUDA_ID, idAyuda);
    }

    cabecera.append(celda);
  });

  activarRedimensionColumnas(cabecera, COLUMNAS, ANCHOS_DEFAULT);

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
  // calcular pertenencia a separador — sin este reconstruirTabla()
  // el carril de números/expandir y los colores/bordes de
  // separador de cada fila quedan con los valores de ANTES del
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
      //
      // [FIX] `nuevoOrden` solo trae los ids VISIBLES (obtenerOrdenIds
      // se arma con construirPlanVisual, que omite las filas de
      // cualquier separador contraído — ver Regla 7). Si se
      // reconstruía `perfil.filas` filtrando solo esos ids, las
      // filas ocultas de un separador contraído que no participó
      // del arrastre quedaban afuera para siempre (se "eliminaban").
      // Ahora, al recorrer cada separador contraído dentro del
      // nuevo orden, se reinserta su tramo oculto completo (tomado
      // del array ANTERIOR, mismo orden relativo) justo después.
      const anterior = perfil.filas;

      const porId = new Map(anterior.map((item) => [item.id, item]));

      const nuevasFilas: ItemFilaPerfil[] = [];
      const yaColocados = new Set<string>();

      nuevoOrden.forEach((id) => {
        const item = porId.get(id);

        if (!item || yaColocados.has(id)) {
          return;
        }

        nuevasFilas.push(item);
        yaColocados.add(id);

        if (esSeparador(item) && !item.expandido) {
          const indiceAnterior = anterior.indexOf(item);

          const tramoOculto = obtenerTramoDeSeparador(anterior, indiceAnterior);

          tramoOculto.forEach((fila) => {
            if (!yaColocados.has(fila.id)) {
              nuevasFilas.push(fila);
              yaColocados.add(fila.id);
            }
          });
        }
      });

      perfil.filas = nuevasFilas;

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
    // el botón ↴/≫ de crearExpandirSeparador) para que las filas que
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

    const numerosAbsolutos = construirNumerosAbsolutos(perfil);

    // Etapa B: el carril solo se ensancha (espacio para el botón
    // Expandir/Contraer) cuando hay al menos un separador visible.
    carrilNumeros.classList.toggle(
      "tiene-separadores",
      plan.some((item) => item.tipo === "separador"),
    );

    // Regla 18 (revisada): el número de fila es su posición
    // absoluta entre TODAS las filas del perfil (contando también
    // las ocultas dentro de separadores contraídos), no la posición
    // entre las filas visibles — así el número que aparece en un
    // mensaje de alerta sigue correspondiendo a la misma fila
    // aunque haya separadores contraídos antes de ella.
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

        const numero = document.createElement("div");

        numero.className = "carril-numero";
        numero.textContent = String(numerosAbsolutos.get(item.fila.id));

        // Etapa C: On/Off superpuesto al número (absolute, ver
        // .carril-numero .estado-toggle en styl_tabla.css).
        numero.append(crearEstado(item.fila, alModificar));

        carrilLista.append(numero);
      } else {
        const headerElemento = crearSeparadorHeader(
          item.separador,
          alModificar,
        );

        filas.append(headerElemento);

        const botonEstadoSeparador = crearEstadoSeparador(
          item.separador,
          alModificar,
        );

        carrilLista.append(
          crearExpandirSeparador(
            item.separador,
            alModificar,
            botonEstadoSeparador,
          ),
        );

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

    actualizarRecorte();
  };

  reconstruirTabla();

  // ==================================================
  // RECONSTRUIR FILA
  // ==================================================

  const reconstruirFila = (id: string): void => {
    const perfil = obtenerPerfilUi();

    const item = perfil.filas.find((fila) => fila.id === id);

    // reconstruirFila es solo para filas normales — un separador
    // se re-renderiza a través de reconstruirTabla (ver
    // crearSeparadorHeader/crearExpandirSeparador), no acá.
    if (!item || esSeparador(item)) {
      return;
    }

    const filaActual = filas.querySelector(`[data-id="${id}"]`);

    if (!filaActual) {
      return;
    }

    // [FIX] Antes se llamaba crearFila() sin el 4º parámetro
    // (info de separador: color/primera/ultima) — la fila
    // reconstruida perdía el color de fondo/borde del separador hasta
    // que otra acción disparara un reconstruirTabla() completo. Se
    // recalcula acá el mismo plan visual que usa reconstruirTabla,
    // así una fila individual queda idéntica a como saldría en un
    // render completo.
    const plan = construirPlanVisual(perfil);

    const planItem = plan.find((p) => p.tipo === "fila" && p.fila.id === id);

    if (!planItem || planItem.tipo !== "fila") {
      return;
    }

    let ultimoIndiceFilaEnPlan = -1;

    plan.forEach((p, i) => {
      if (p.tipo === "fila") {
        ultimoIndiceFilaEnPlan = i;
      }
    });

    const indicePlan = plan.indexOf(planItem);

    const esUltima = planItem.separador
      ? planItem.separador.ultima
      : indicePlan === ultimoIndiceFilaEnPlan;

    const filaNueva = crearFila(
      planItem.fila,
      esUltima,
      alModificar,
      planItem.separador,
    );

    filaActual.replaceWith(filaNueva);

    registrarFilaArrastrable(filaNueva);

    // [FIX bug 2] El botón On/Off con estado de alerta vive
    // superpuesto al número, en el carril (carrilLista), no dentro
    // de la fila reconstruida arriba — sin este paso, una fila que
    // entra/sale de conflicto o advertencia no actualizaba su
    // círculo hasta el próximo reconstruirTabla() completo. Se
    // ubica el mismo índice visual en carrilLista (incluye slots de
    // separador) y se reemplaza solo su .estado-toggle.
    const indiceCarril = plan.indexOf(planItem);

    const numeroActual = carrilLista.children[indiceCarril] as
      | HTMLElement
      | undefined;

    if (numeroActual?.classList.contains("carril-numero")) {
      const botonViejo =
        numeroActual.querySelector<HTMLElement>(".estado-toggle");

      const botonNuevo = crearEstado(planItem.fila, alModificar);

      if (botonViejo) {
        numeroActual.replaceChild(botonNuevo, botonViejo);
      } else {
        numeroActual.append(botonNuevo);
      }
    }
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
  });

  viewport.append(cabecera, filas);

  tabla.append(carrilNumeros, viewport);

  // Actualiza el botón estado de los separadores que contienen alguna
  // de las filas afectadas, sin reconstruir la tabla entera (bug 2).
  const actualizarSeparadoresDeFilas = (idsFilas: string[]): void => {
    const perfil = obtenerPerfilUi();

    const separadoresActualizados = new Set<string>();

    for (const idFila of idsFilas) {
      // Buscar el separador padre de esta fila en el modelo
      let separadorPadre: SeparadorPerfil | null = null;

      for (let i = 0; i < perfil.filas.length; i++) {
        const item = perfil.filas[i];

        if (esSeparador(item)) {
          const tramo = obtenerTramoDeSeparador(perfil.filas, i);

          if (tramo.some((f) => f.id === idFila)) {
            separadorPadre = item;

            break;
          }
        }
      }

      if (!separadorPadre || separadoresActualizados.has(separadorPadre.id)) {
        continue;
      }

      separadoresActualizados.add(separadorPadre.id);

      // Buscar el slot Expandir/On-Off del separador en el carril
      // (ya no vive en el header, ver Etapa D) y reemplazar solo el
      // botón estado (primer hijo del slot).
      const slotEl = carrilLista.querySelector<HTMLElement>(
        `.carril-expandir-slot[data-id="${separadorPadre.id}"]`,
      );

      if (!slotEl) {
        continue;
      }

      const botonViejo = slotEl.querySelector<HTMLElement>(".estado-toggle");

      const botonNuevo = crearEstadoSeparador(separadorPadre, alModificar);

      if (botonViejo?.parentElement) {
        botonViejo.parentElement.replaceChild(botonNuevo, botonViejo);
      } else {
        const wrapperEstado = slotEl.querySelector<HTMLElement>(
          ".carril-expandir-slot-numero",
        );

        wrapperEstado?.append(botonNuevo);
      }
    }
  };

  registrarReconstruccion(
    reconstruirTabla,
    reconstruirFila,
    actualizarSeparadoresDeFilas,
  );

  return tabla;
}
