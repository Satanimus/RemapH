// ======================================================
// 🔘 comp_Popup_Grupo
// ------------------------------------------------------
// Piezas compartidas para popups persistentes (los que se
// redibujan sin cerrarse, ver comp_popup_coordenada.ts):
// grupo de opciones tipo-radio, fila con etiqueta, interruptor
// deslizante independiente, y el indicador cyan que marca cuál
// opción está activa (o es la default sin tocar) en el grupo.
// Antes esto vivía duplicado dentro de comp_popup_coordenada.ts;
// se extrae acá para que cualquier popup nuevo con este mismo
// patrón lo reuse en vez de reimplementarlo.
// ======================================================

// ======================================================
// 🔵 INDICADOR ACTIVO (círculo cyan)
// ======================================================

export function crearIndicadorActivo(): HTMLSpanElement {
  const indicador = document.createElement("span");

  indicador.className = "popup-indicador-activo";

  return indicador;
}

// ======================================================
// 🔘 GRUPO DE OPCIONES (fila de botones tipo radio)
// ======================================================

export function crearGrupoOpciones<T extends string>(
  opciones: {
    texto: string;
    valor: T;
    // Opcionales — para grupos donde alguna opción puede quedar
    // inhabilitada según el estado de la fila (ej. "En App" en el
    // popup Extra de Multimedia, ver comp_popup_multimedia_extra.ts).
    // `titulo` es el tooltip que explica el porqué mientras está
    // deshabilitada.
    deshabilitado?: boolean;
    titulo?: string;
  }[],
  valorActual: T,
  onSeleccionar: (valor: T) => void,
  claseExtra?: string,
): HTMLElement {
  const grupo = document.createElement("div");

  grupo.className = claseExtra ? `popup-grupo ${claseExtra}` : "popup-grupo";

  opciones.forEach((opcion) => {
    const boton = document.createElement("button");

    boton.className = "ui-btn popup-opcion";

    const activo = opcion.valor === valorActual;

    boton.dataset.activo = activo ? "true" : "false";

    if (activo) {
      boton.append(crearIndicadorActivo());
    }

    boton.append(document.createTextNode(opcion.texto));

    if (opcion.deshabilitado) {
      boton.disabled = true;

      if (opcion.titulo) {
        boton.title = opcion.titulo;
      }
    } else {
      boton.addEventListener("click", () => {
        onSeleccionar(opcion.valor);
      });
    }

    grupo.append(boton);
  });

  return grupo;
}

// ======================================================
// 🔀 INTERRUPTOR (switch deslizante, independiente de grupo)
// ------------------------------------------------------
// Para casos como Coordenada: una opción que se prende o
// apaga por sí misma, sin excluir otras. Se ve como un
// switch tipo iOS/Android — pista + bolita que se desliza —
// en vez del botón con indicador cyan usado en los grupos
// tipo-radio, para que quede claro a simple vista que es un
// ON/OFF y no una elección entre varias.
// ======================================================

export function crearInterruptor(
  texto: string,
  activo: boolean,
  onClick: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn popup-switch";

  boton.dataset.activo = activo ? "true" : "false";

  const pista = document.createElement("span");

  pista.className = "popup-switch-pista";

  const bolita = document.createElement("span");

  bolita.className = "popup-switch-bolita";

  pista.append(bolita);

  boton.append(pista, document.createTextNode(texto));

  boton.addEventListener("click", onClick);

  return boton;
}

// ======================================================
// 🏷️ FILA CON ETIQUETA
// ======================================================

export function crearFilaPopup(
  etiqueta: string,
  contenido: HTMLElement,
): HTMLElement {
  const fila = document.createElement("div");

  fila.className = "popup-fila";

  const label = document.createElement("span");

  label.className = "popup-fila-label";

  label.textContent = etiqueta;

  fila.append(label, contenido);

  return fila;
}
