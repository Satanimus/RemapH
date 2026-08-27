// ======================================================
// ❔📝 util_Texto_Ayuda
// ------------------------------------------------------
// Parser de las marcas de formato del contenido de ayuda.txt
// (Regla 9): **negrita**, *cursiva*, `código`, [cyan:texto],
// [red:texto] — sin HTML, todo armado vía DOM.
// ======================================================

const REGEX_MARCAS =
  /\*\*(.+?)\*\*|\*(.+?)\*|`(.+?)`|\[cyan:(.+?)\]|\[red:(.+?)\]/g;

export function parsearLineaAyuda(linea: string): Node[] {
  const nodos: Node[] = [];

  let ultimoIndice = 0;

  let coincidencia: RegExpExecArray | null;

  REGEX_MARCAS.lastIndex = 0;

  while ((coincidencia = REGEX_MARCAS.exec(linea)) !== null) {
    if (coincidencia.index > ultimoIndice) {
      nodos.push(
        document.createTextNode(linea.slice(ultimoIndice, coincidencia.index)),
      );
    }

    const [, negrita, cursiva, codigo, cyan, red] = coincidencia;

    if (negrita !== undefined) {
      const elemento = document.createElement("strong");

      elemento.textContent = negrita;

      nodos.push(elemento);
    } else if (cursiva !== undefined) {
      const elemento = document.createElement("em");

      elemento.textContent = cursiva;

      nodos.push(elemento);
    } else if (codigo !== undefined) {
      const elemento = document.createElement("code");

      elemento.textContent = codigo;

      nodos.push(elemento);
    } else if (cyan !== undefined) {
      const elemento = document.createElement("span");

      elemento.className = "ayuda-cyan";

      elemento.textContent = cyan;

      nodos.push(elemento);
    } else if (red !== undefined) {
      const elemento = document.createElement("span");

      elemento.className = "ayuda-red";

      elemento.textContent = red;

      nodos.push(elemento);
    }

    ultimoIndice = REGEX_MARCAS.lastIndex;
  }

  if (ultimoIndice < linea.length) {
    nodos.push(document.createTextNode(linea.slice(ultimoIndice)));
  }

  return nodos;
}

export function renderizarAyuda(texto: string): DocumentFragment {
  const fragmento = document.createDocumentFragment();

  texto.split("\n").forEach((linea) => {
    const div = document.createElement("div");

    div.className = "ayuda-linea";

    div.append(...parsearLineaAyuda(linea));

    fragmento.append(div);
  });

  return fragmento;
}
