// ======================================================
// 🪟 comp_Popup_Contenedor
// ------------------------------------------------------
// Capa compartida por TODOS los popups.
//
// Cierra solo con click en el fondo, nunca por burbujeo
// desde el contenido (inputs, botones, etc), salvo los
// popups montados con mostrarPopupFijo() (ver más abajo),
// que directamente ignoran el click en el fondo.
// ======================================================

let capaPopup: HTMLElement | null = null;
let alCerrarActual: (() => void) | null = null;
let popupFijoActual = false;

export function crearContenedorPopup(): HTMLElement {
  if (capaPopup) {
    return capaPopup;
  }

  capaPopup = document.createElement("div");

  capaPopup.className = "popup-capa";

  capaPopup.addEventListener("click", (evento) => {
    if (popupFijoActual) {
      return;
    }

    if (evento.target === capaPopup) {
      ocultarPopup();
    }
  });

  return capaPopup;
}

export function mostrarPopup(
  contenido: HTMLElement,
  x?: number,
  y?: number,
  alCerrar?: () => void,
): void {
  mostrarPopupInterno(contenido, x, y, alCerrar, false);
}

// ======================================================
// 📌 POPUP FIJO
// ------------------------------------------------------
// Mismo montaje que mostrarPopup(), pero el click en el fondo
// de la capa NO lo cierra (el popup solo se cierra mediante
// una acción explícita del propio contenido, p. ej. un botón
// Cancelar/Guardar). Usado por el editor de Macro
// (comp_popup_macro_editor.ts), que además es arrastrable.
// ======================================================

export function mostrarPopupFijo(
  contenido: HTMLElement,
  x?: number,
  y?: number,
  alCerrar?: () => void,
): void {
  mostrarPopupInterno(contenido, x, y, alCerrar, true);
}

function mostrarPopupInterno(
  contenido: HTMLElement,
  x: number | undefined,
  y: number | undefined,
  alCerrar: (() => void) | undefined,
  fijo: boolean,
): void {
  if (!capaPopup) {
    return;
  }

  capaPopup.innerHTML = "";

  capaPopup.append(contenido);

  capaPopup.style.display = "block";

  alCerrarActual = alCerrar ?? null;

  popupFijoActual = fijo;

  if (x !== undefined && y !== undefined) {
    contenido.style.position = "fixed";
    contenido.style.left = `${x}px`;
    contenido.style.top = `${y}px`;
    contenido.style.right = "";
    contenido.style.bottom = "";

    ajustarPosicionDentroDeVentana(contenido, x, y);
  } else {
    contenido.style.position = "static";
  }
}

// Elige la esquina del popup usada como referencia (izquierda/derecha,
// arriba/abajo) según cuál deja el contenido completo dentro de la
// ventana. Si el popup es más grande que el espacio disponible en algún
// eje, se lo pega directo al borde correspondiente.
function ajustarPosicionDentroDeVentana(
  contenido: HTMLElement,
  x: number,
  y: number,
): void {
  const margen = 4;
  const anchoVentana = window.innerWidth;
  const altoVentana = window.innerHeight;
  const ancho = contenido.offsetWidth;
  const alto = contenido.offsetHeight;

  if (x + ancho > anchoVentana - margen) {
    contenido.style.left = "";
    contenido.style.right = `${Math.max(margen, anchoVentana - x)}px`;
  }

  if (y + alto > altoVentana - margen) {
    contenido.style.top = "";
    contenido.style.bottom = `${Math.max(margen, altoVentana - y)}px`;
  }

  const rect = contenido.getBoundingClientRect();

  if (rect.left < margen) {
    contenido.style.left = `${margen}px`;
    contenido.style.right = "";
  }

  if (rect.top < margen) {
    contenido.style.top = `${margen}px`;
    contenido.style.bottom = "";
  }
}

export function ocultarPopup(): void {
  if (!capaPopup) {
    return;
  }

  capaPopup.style.display = "none";

  popupFijoActual = false;

  const alCerrar = alCerrarActual;

  alCerrarActual = null;

  alCerrar?.();
}
