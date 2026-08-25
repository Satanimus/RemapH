// ======================================================
// 🗂️ core_Separadores
// ------------------------------------------------------
// Lógica pura de pertenencia entre filas y Separadores.
// No toca el DOM ni el estado global: recibe datos, devuelve datos.
// En el nuevo modelo, los separadores son elementos del array
// `perfil.filas` (tipo "separador"). La pertenencia se deriva
// por posición en cada render (Regla 3).
// ======================================================

import type {
  FilaPerfil,
  ItemFilaPerfil,
  Perfil,
  SeparadorPerfil,
} from "./core_perfil";

// ======================================================
// 🧩 TYPE GUARD
// ======================================================

export function esSeparador(item: ItemFilaPerfil): item is SeparadorPerfil {
  return item.tipoItem === "separador";
}

// ======================================================
// 🧩 PLAN VISUAL
// ------------------------------------------------------
// Arma la secuencia real de lo que hay que dibujar:
// separadores y filas (omitidas si están dentro de un separador
// contraído). `ui_tabla.ts` y el carril de números recorren
// este mismo array ítem por ítem.
// ======================================================

export type ItemVisualTabla =
  | { tipo: "separador"; separador: SeparadorPerfil }
  | {
      tipo: "fila";
      fila: FilaPerfil;
      indiceAbsoluto: number;
      separador?: { color: string; primera: boolean; ultima: boolean };
    };

export function construirPlanVisual(perfil: Perfil): ItemVisualTabla[] {
  const plan: ItemVisualTabla[] = [];

  const items = perfil.filas;

  let separadorActivo: SeparadorPerfil | null = null;

  for (let i = 0; i < items.length; i++) {
    const item = items[i];

    if (esSeparador(item)) {
      separadorActivo = item;

      plan.push({ tipo: "separador", separador: item });

      continue;
    }

    // Fila normal (el type guard de arriba ya angostó `item` en el
    // resto del bloque a todo lo que no sea SeparadorPerfil, es
    // decir FilaPerfil — sin necesidad de cast).
    const fila = item;

    const indiceAbsoluto = i;

    // Regla 7: si el separador activo está contraído, la fila no
    // entra al plan visual.
    if (separadorActivo && !separadorActivo.expandido) {
      continue;
    }

    const anteriorEsSeparador = i > 0 && esSeparador(items[i - 1]);

    const siguienteEsSeparador =
      i + 1 < items.length && esSeparador(items[i + 1]);

    const finDeArray = i === items.length - 1;

    const esPrimera = anteriorEsSeparador;

    const esUltima = siguienteEsSeparador || finDeArray;

    // Regla 4: filas antes del primer separador no pertenecen a
    // ninguno, por eso solo se agrega la info si hay separador activo.
    const infoSeparador = separadorActivo
      ? {
          color: separadorActivo.color,

          primera: esPrimera,

          ultima: esUltima,
        }
      : undefined;

    plan.push({
      tipo: "fila",

      fila,

      indiceAbsoluto,

      separador: infoSeparador,
    });
  }

  return plan;
}

export function construirNumerosAbsolutos(perfil: Perfil): Map<string, number> {
  const mapa = new Map<string, number>();

  let contador = 0;

  for (const item of perfil.filas) {
    if (esSeparador(item)) {
      continue;
    }

    contador += 1;

    mapa.set(item.id, contador);
  }

  return mapa;
}

// ======================================================
// 📏 TRAMO DE SEPARADOR
// ------------------------------------------------------
// Filas entre este separador (exclusive) y el siguiente
// separador o el fin del array (exclusive), sin importar
// si `expandido` está en false — el tramo existe igual,
// solo no se dibuja (ver construirPlanVisual).
// ======================================================

export function obtenerTramoDeSeparador(
  filas: ItemFilaPerfil[],
  indiceSeparador: number,
): FilaPerfil[] {
  const tramo: FilaPerfil[] = [];

  for (let i = indiceSeparador + 1; i < filas.length; i++) {
    const item = filas[i];

    if (esSeparador(item)) break;

    tramo.push(item);
  }

  return tramo;
}

// ======================================================
// ⬇️ CASCADA DESCENDENTE
// ------------------------------------------------------
// Regla 14: al cambiar manualmente el estado ON/OFF de un
// separador, sobrescribe el estado de todas las filas de
// su tramo (hasta el siguiente separador o fin de array).
// ======================================================

export function aplicarCascadaDescendente(
  filas: ItemFilaPerfil[],
  indiceSeparador: number,
  nuevoEstado: string,
): void {
  for (let i = indiceSeparador + 1; i < filas.length; i++) {
    const item = filas[i];

    if (esSeparador(item)) break;

    item.estado = nuevoEstado;
  }
}

// ======================================================
// ⬆️ CASCADA ASCENDENTE (RECOMPUTACIÓN)
// ------------------------------------------------------
// Regla 15/16: recorre todo el array por tramos y recalcula
// `estadoVisual` de cada separador según el estado real de
// las filas de su tramo. No toca `estado` (último ON/OFF
// explícito del separador, usado por la cascada descendente).
// ======================================================

export function recomputarCascadaAscendente(filas: ItemFilaPerfil[]): void {
  for (let i = 0; i < filas.length; i++) {
    const item = filas[i];

    if (!esSeparador(item)) continue;

    const tramo = obtenerTramoDeSeparador(filas, i);

    item.estadoVisual = estadoSeparadorVigente(item, tramo);
  }
}

// ======================================================
// 🔴🟢 ESTADO VIGENTE DEL SEPARADOR
// ------------------------------------------------------
// Compara el estado guardado del separador contra el estado
// real de sus filas contenidas (el tramo que le corresponde).
// Usado por comp_separador_estado.ts para decidir si mostrar
// el indicador gris (mixto).
// ======================================================

export function estadoSeparadorVigente(
  separador: SeparadorPerfil,
  filasDelTramo: FilaPerfil[],
): "on" | "off" | "mixto" {
  if (filasDelTramo.length === 0) {
    return separador.estado === "ON" ? "on" : "off";
  }

  let hayOn = false;

  let hayOff = false;

  for (const fila of filasDelTramo) {
    if (fila.estado === "ON") hayOn = true;
    else hayOff = true;

    if (hayOn && hayOff) return "mixto";
  }

  return hayOn ? "on" : "off";
}
