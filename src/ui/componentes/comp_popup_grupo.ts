// ======================================================
// 🔘 comp_Popup_Grupo
// ------------------------------------------------------
// Piezas compartidas para popups persistentes (los que se
// redibujan sin cerrarse, ver comp_popup_coordenada.ts):
// grupo de opciones tipo-radio, fila con etiqueta, botón
// toggle independiente, y el indicador cyan que marca cuál
// opción está activa (o es la default sin tocar) en
// cualquiera de los dos. Antes esto vivía duplicado dentro
// de comp_popup_coordenada.ts; se extrae acá para que
// cualquier popup nuevo con este mismo patrón lo reuse en
// vez de reimplementarlo.
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
  opciones: { texto: string; valor: T }[],
  valorActual: T,
  onSeleccionar: (valor: T) => void,
): HTMLElement {
  const grupo = document.createElement("div");

  grupo.className = "popup-grupo";

  opciones.forEach((opcion) => {
    const boton = document.createElement("button");

    boton.className = "ui-btn popup-opcion";

    const activo = opcion.valor === valorActual;

    boton.dataset.activo = activo ? "true" : "false";

    if (activo) {
      boton.append(crearIndicadorActivo());
    }

    boton.append(document.createTextNode(opcion.texto));

    boton.addEventListener("click", () => {
      onSeleccionar(opcion.valor);
    });

    grupo.append(boton);
  });

  return grupo;
}

// ======================================================
// 🔘 BOTÓN TOGGLE (independiente, no forma parte de un grupo)
// ------------------------------------------------------
// Para casos como Coordenada: un único botón que se prende o
// apaga por sí mismo, sin excluir otras opciones. Usa el
// mismo indicador que el grupo, para que la identidad visual
// sea consistente en todo el popup.
// ======================================================

export function crearBotonToggle(
  texto: string,
  activo: boolean,
  onClick: () => void,
  indicadorPersonalizado?: string,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn popup-opcion popup-toggle";

  boton.dataset.activo = activo ? "true" : "false";

  if (activo) {
    if (indicadorPersonalizado) {
      const indicador = document.createElement("span");

      indicador.className = "popup-indicador-personalizado";

      indicador.textContent = indicadorPersonalizado;

      boton.append(indicador);
    } else {
      boton.append(crearIndicadorActivo());
    }
  }

  boton.append(document.createTextNode(texto));

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
