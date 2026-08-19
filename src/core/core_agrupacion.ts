// ======================================================
// 🗂️ core_Agrupacion
// ------------------------------------------------------
// Lógica pura de pertenencia entre filas y Agrupaciones.
// No toca el DOM ni el estado global (obtenerPerfilUi()):
// recibe datos, devuelve datos. La usan Etapa B (render)
// y Etapa D (arrastre) para no duplicar el cálculo de rangos.
// ======================================================

import type { AgrupacionPerfil, FilaPerfil, Perfil } from "./core_perfil";

// ======================================================
// 📐 PERTENENCIA
// ======================================================

export interface RangoGrupo {
  inicio: number;

  fin: number;
}

export interface Pertenencia {
  // Índice de fila (0-based, sobre el array `filas`) -> id del
  // grupo al que pertenece, o null si es una fila suelta.
  filaAGrupo: (string | null)[];

  // id de grupo -> rango [inicio, fin) de índices de fila que contiene.
  rangoPorGrupo: Map<string, RangoGrupo>;
}

// ======================================================
// 📐 CALCULAR PERTENENCIA
// ======================================================

export function calcularPertenencia(
  grupos: AgrupacionPerfil[],
  totalFilas: number,
): Pertenencia {
  const filaAGrupo: (string | null)[] = new Array(totalFilas).fill(null);

  const rangoPorGrupo = new Map<string, RangoGrupo>();

  let cursor = 0;

  for (const grupo of grupos) {
    const inicio = cursor;

    const fin = Math.min(cursor + grupo.numFilas, totalFilas);

    rangoPorGrupo.set(grupo.id, { inicio, fin });

    for (let i = inicio; i < fin; i++) {
      filaAGrupo[i] = grupo.id;
    }

    cursor = fin;
  }

  // Lo que sobra después de sumar todos los grupos queda sin
  // grupo (filaAGrupo ya vale null ahí por el fill inicial).

  return { filaAGrupo, rangoPorGrupo };
}

// ======================================================
// 🔄 RECALCULAR GRUPOS
// ======================================================

// Dada la lista final ya reordenada (una secuencia de ids que
// mezcla ids de grupo e ids de fila, en el orden visual tras un
// movimiento), devuelve un nuevo array `grupos` con los `numFilas`
// recalculados. Cada tramo de ids de fila entre un id de grupo y
// el siguiente (o el final) pasa a ser el `numFilas` de ese grupo;
// lo que sobra después del último grupo son las filas sueltas y
// queda fuera del resultado.
//
// `filaAGrupoAntes` es la pertenencia (id de fila -> id de grupo
// anterior, o null si estaba suelta) DE ANTES de este reordenamiento.
// Hace falta porque una fila suelta que quedó, por cualquier otro
// arrastre ajeno al último grupo, justo después de él en
// `ordenVisual` es indistinguible -por posición- de una fila que el
// usuario sí arrastró adentro: sin esta pista, el último grupo se
// come todas las filas sueltas que queden por debajo (bug: "el
// grupo inferior se expande y toma todas las filas sueltas"). Las
// filas sueltas que mantienen su orden relativo de antes (subsecuencia
// común más larga contra el orden viejo) se consideran "no tocadas"
// y no pasan a ningún grupo; las que rompen ese orden relativo se
// interpretan como recién arrastradas y sí se cuentan.
export function recalcularGrupos(
  grupos: AgrupacionPerfil[],
  ordenVisual: string[],
  filaAGrupoAntes: Map<string, string | null>,
): AgrupacionPerfil[] {
  const grupoPorId = new Map(grupos.map((g) => [g.id, g]));

  const sueltasAntes = [...filaAGrupoAntes.entries()]
    .filter(([, g]) => g === null)
    .map(([id]) => id);

  const colaSueltasCandidatas = ordenVisual.filter(
    (id) => !grupoPorId.has(id) && filaAGrupoAntes.get(id) === null,
  );

  const siguenSueltas = idsQueMantienenOrdenRelativo(
    colaSueltasCandidatas,
    sueltasAntes,
  );

  const resultado: AgrupacionPerfil[] = [];

  let grupoActual: AgrupacionPerfil | null = null;

  for (const id of ordenVisual) {
    const grupo = grupoPorId.get(id);

    if (grupo) {
      grupoActual = { ...grupo, numFilas: 0 };

      resultado.push(grupoActual);

      continue;
    }

    if (!grupoActual) {
      // Fila suelta antes de cualquier header: no cuenta para nadie.
      continue;
    }

    if (siguenSueltas.has(id)) {
      // Suelta de antes que no cambió de orden relativo respecto a
      // las demás sueltas: sigue suelta, no se une al grupo actual.
      continue;
    }

    grupoActual.numFilas += 1;
  }

  return resultado;
}

// Subsecuencia común más larga (por identidad de id, no de valor)
// entre `nuevoOrden` y `ordenAnterior`: los ids que la integran son
// los que NO cambiaron de posición relativa entre sí. Se resuelve
// como longest-increasing-subsequence sobre los índices de
// `nuevoOrden` dentro de `ordenAnterior`.
function idsQueMantienenOrdenRelativo(
  nuevoOrden: string[],
  ordenAnterior: string[],
): Set<string> {
  const posicionAnterior = new Map(ordenAnterior.map((id, i) => [id, i]));

  const indices = nuevoOrden.map((id) => posicionAnterior.get(id)!);

  const n = indices.length;

  const largoHasta = new Array<number>(n).fill(1);

  const anteriorEnCadena = new Array<number>(n).fill(-1);

  let mejorFinal = -1;

  for (let i = 0; i < n; i++) {
    for (let j = 0; j < i; j++) {
      if (indices[j] < indices[i] && largoHasta[j] + 1 > largoHasta[i]) {
        largoHasta[i] = largoHasta[j] + 1;

        anteriorEnCadena[i] = j;
      }
    }

    if (mejorFinal === -1 || largoHasta[i] > largoHasta[mejorFinal]) {
      mejorFinal = i;
    }
  }

  const resultado = new Set<string>();

  let cursor = mejorFinal;

  while (cursor !== -1) {
    resultado.add(nuevoOrden[cursor]);

    cursor = anteriorEnCadena[cursor];
  }

  return resultado;
}

// ======================================================
// 🧩 PLAN VISUAL
// ------------------------------------------------------
// Arma, en un solo array ordenado, la secuencia real de lo que
// hay que dibujar: headers de grupo y filas, ya filtrando las
// filas internas de los grupos colapsados. `ui_tabla.ts` (tabla)
// y el carril de números recorren este mismo array, ítem por
// ítem, para quedar sincronizados 1 a 1.
// ======================================================

export type ItemVisualTabla =
  | { tipo: "grupo"; grupo: AgrupacionPerfil }
  | {
      tipo: "fila";
      fila: FilaPerfil;
      indiceAbsoluto: number;
      grupo?: { color: string; primera: boolean; ultima: boolean };
    }
  | { tipo: "placeholder"; grupo: AgrupacionPerfil };

export function construirPlanVisual(perfil: Perfil): ItemVisualTabla[] {
  const { filaAGrupo, rangoPorGrupo } = calcularPertenencia(
    perfil.grupos,
    perfil.filas.length,
  );

  const plan: ItemVisualTabla[] = [];

  for (const grupo of perfil.grupos) {
    plan.push({ tipo: "grupo", grupo });

    if (!grupo.expandido) {
      continue;
    }

    const rango = rangoPorGrupo.get(grupo.id);

    if (!rango) {
      continue;
    }

    if (rango.fin === rango.inicio) {
      plan.push({ tipo: "placeholder", grupo });

      continue;
    }

    for (let i = rango.inicio; i < rango.fin; i++) {
      plan.push({
        tipo: "fila",
        fila: perfil.filas[i],
        indiceAbsoluto: i,
        grupo: {
          color: grupo.color,
          primera: i === rango.inicio,
          ultima: i === rango.fin - 1,
        },
      });
    }
  }

  for (let i = 0; i < perfil.filas.length; i++) {
    if (filaAGrupo[i] === null) {
      plan.push({ tipo: "fila", fila: perfil.filas[i], indiceAbsoluto: i });
    }
  }

  return plan;
}

// ======================================================
// 🔴🟢 ESTADO VIGENTE DEL GRUPO
// ------------------------------------------------------
// Compara el estado guardado del grupo contra el estado real
// de sus filas contenidas. Usado por comp_grupo_estado.ts
// para decidir si mostrar el indicador gris (mixto).
// ======================================================

export function estadoGrupoVigente(
  grupo: AgrupacionPerfil,
  filas: FilaPerfil[],
  rango: RangoGrupo,
): "on" | "off" | "mixto" {
  if (rango.inicio === rango.fin) {
    return grupo.estado === "ON" ? "on" : "off";
  }

  for (let i = rango.inicio; i < rango.fin; i++) {
    if (filas[i].estado !== grupo.estado) {
      return "mixto";
    }
  }

  return grupo.estado === "ON" ? "on" : "off";
}
