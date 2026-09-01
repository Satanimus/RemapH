// ======================================================
// ⁝⁝ util_DuplicarSeleccion
// ------------------------------------------------------
// Utilidad GENÉRICA para "Duplicar seleccionados" en
// cualquier tabla de filas reordenables. No conoce el
// contenido de cada fila — recibe el arreglo, los ids
// seleccionados, cómo obtener el id de un elemento y cómo
// clonarlo, y devuelve el arreglo resultante con los
// duplicados agrupados en bloque, en vez de intercalados
// uno a uno junto a su original.
//
// Regla: el bloque de copias se inserta completo
// inmediatamente después del ÚLTIMO elemento seleccionado
// (en orden de aparición actual en el arreglo), y las
// copias dentro del bloque conservan ese mismo orden
// relativo.
//
// Ejemplo con selección de las filas 1, 3 y 5 (0-index):
//   Antes:    [A, B*, C, D*, E, F*, G]
//   Después:  [A, B*, C, D*, E, F*, B', D', F', G]
// (uno-a-uno daría [A, B*, B', C, D*, D', E, F*, F', G] —
// intercalado, que es justo lo que esto evita).
// ======================================================

export function duplicarSeleccionComoBloque<T>(
  elementos: T[],
  idsSeleccionados: Iterable<string>,
  obtenerId: (elemento: T) => string,
  clonar: (elemento: T, indiceEnBloque: number) => T,
): T[] {
  const idsASet = new Set(idsSeleccionados);

  if (idsASet.size === 0) {
    return elementos.slice();
  }

  // Índices seleccionados, en el orden actual del arreglo.
  const indicesSeleccionados = elementos.reduce<number[]>((acc, elemento, indice) => {
    if (idsASet.has(obtenerId(elemento))) {
      acc.push(indice);
    }
    return acc;
  }, []);

  if (indicesSeleccionados.length === 0) {
    return elementos.slice();
  }

  const indiceInsercion = indicesSeleccionados[indicesSeleccionados.length - 1] + 1;

  const bloqueCopias = indicesSeleccionados.map((indiceOriginal, indiceEnBloque) =>
    clonar(elementos[indiceOriginal], indiceEnBloque),
  );

  const resultado = elementos.slice();
  resultado.splice(indiceInsercion, 0, ...bloqueCopias);

  return resultado;
}
