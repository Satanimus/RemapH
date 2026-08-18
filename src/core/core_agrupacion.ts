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
// No necesita saber nada de FilaPerfil: solo distingue si un id de
// `ordenVisual` es un grupo conocido (está en `grupos`) o no.
export function recalcularGrupos(
  grupos: AgrupacionPerfil[],
  ordenVisual: string[],
): AgrupacionPerfil[] {
  const grupoPorId = new Map(grupos.map((g) => [g.id, g]));

  const resultado: AgrupacionPerfil[] = [];

  let grupoActual: AgrupacionPerfil | null = null;

  for (const id of ordenVisual) {
    const grupo = grupoPorId.get(id);

    if (grupo) {
      grupoActual = { ...grupo, numFilas: 0 };

      resultado.push(grupoActual);
    } else if (grupoActual) {
      grupoActual.numFilas += 1;
    }

    // Si no hay grupoActual todavía y el id no es de grupo, es una
    // fila suelta antes de cualquier header: no cuenta para nadie.
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
        grupo: grupo.color
          ? {
              color: grupo.color,
              primera: i === rango.inicio,
              ultima: i === rango.fin - 1,
            }
          : undefined,
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
