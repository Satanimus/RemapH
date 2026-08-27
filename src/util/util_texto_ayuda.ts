// ======================================================
// ❔📝 util_Texto_Ayuda
// ------------------------------------------------------
// Parser de las marcas de formato del contenido de ayuda.txt
// (Regla 9): **negrita**, *cursiva*, `código`, [color:texto]
// (cyan, red, orange, yellow, green, blue, purple, pink, gray)
// — sin HTML, todo armado vía DOM. Las marcas [color:...] son
// recursivas: **negrita**/*cursiva*/`código` dentro de un color
// se combinan (ej. [green:*Activo*] queda verde + cursiva).
// ======================================================

const PATRON_MARCAS =
  /\*\*(.+?)\*\*|\*(.+?)\*|`(.+?)`|\[cyan:(.+?)\]|\[red:(.+?)\]|\[(orange|yellow|green|blue|purple|pink|gray):(.+?)\]/g;

export function parsearLineaAyuda(linea: string): Node[] {
  const nodos: Node[] = [];

  let ultimoIndice = 0;

  let coincidencia: RegExpExecArray | null;

  // Instancia propia por llamada: al ser recursiva (marcas de
  // color anidan negrita/cursiva/código), un regex /g compartido
  // a nivel de módulo se pisa entre la llamada externa y la
  // interna (comparten lastIndex), lo que podía dejar el `while`
  // externo sin avanzar nunca — congelando la interfaz.
  const regex = new RegExp(PATRON_MARCAS.source, "g");

  while ((coincidencia = regex.exec(linea)) !== null) {
    if (coincidencia.index > ultimoIndice) {
      nodos.push(
        document.createTextNode(linea.slice(ultimoIndice, coincidencia.index)),
      );
    }

    const [, negrita, cursiva, codigo, cyan, red, colorNombre, colorTexto] =
      coincidencia;

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

      elemento.append(...parsearLineaAyuda(cyan));

      nodos.push(elemento);
    } else if (red !== undefined) {
      const elemento = document.createElement("span");

      elemento.className = "ayuda-red";

      elemento.append(...parsearLineaAyuda(red));

      nodos.push(elemento);
    } else if (colorNombre !== undefined) {
      const elemento = document.createElement("span");

      elemento.className = `ayuda-${colorNombre}`;

      elemento.append(...parsearLineaAyuda(colorTexto));

      nodos.push(elemento);
    }

    ultimoIndice = regex.lastIndex;
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
