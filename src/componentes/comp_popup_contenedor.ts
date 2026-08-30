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
let origenActual: HTMLElement | null = null;

// Último botón (.ui-btn) sobre el que se hizo mousedown —
// capturado globalmente para no tener que cambiar todas las
// llamadas a mostrarPopup que ya pasan solo coordenadas.
let ultimoBotonPulsado: HTMLElement | null = null;

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

  document.addEventListener("keydown", (evento) => {
    if (evento.key !== "Escape") return;
    if (popupFijoActual) return;
    if (capaPopup?.style.display === "none") return;

    evento.stopPropagation();
    ocultarPopup();
  });

  // Captura el botón origen antes de que mostrarPopup se llame
  document.addEventListener(
    "mousedown",
    (evento) => {
      const el = evento.target as HTMLElement;
      ultimoBotonPulsado = el.closest(".ui-btn");
    },
    true,
  );

  return capaPopup;
}

export function mostrarPopup(
  contenido: HTMLElement,
  x?: number,
  y?: number,
  alCerrar?: () => void,
  origen?: HTMLElement,
): void {
  mostrarPopupInterno(contenido, x, y, alCerrar, false, origen);
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
  origen?: HTMLElement,
): void {
  mostrarPopupInterno(contenido, x, y, alCerrar, true, origen);
}

function mostrarPopupInterno(
  contenido: HTMLElement,
  x: number | undefined,
  y: number | undefined,
  alCerrar: (() => void) | undefined,
  fijo: boolean,
  origen: HTMLElement | undefined,
): void {
  if (!capaPopup) {
    return;
  }

  // Limpiar origen anterior
  if (origenActual) {
    origenActual.dataset.abierto = "false";
    origenActual = null;
  }

  capaPopup.innerHTML = "";

  capaPopup.append(contenido);

  capaPopup.style.display = "block";

  alCerrarActual = alCerrar ?? null;

  popupFijoActual = fijo;

  // Usar el origen explícito o el último botón pulsado
  origenActual = origen ?? ultimoBotonPulsado ?? null;

  if (origenActual) {
    origenActual.dataset.abierto = "true";
  }

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

// ======================================================
// 🔄 ACTUALIZAR CONTENIDO SIN REPOSICIONAR
// ------------------------------------------------------
// Reemplaza el contenido de un popup ya abierto conservando
// exactamente la posición (left/top/right/bottom) ya calculada
// en el mostrarPopup() original. Para popups que se redibujan a
// sí mismos al cambiar una opción (p. ej. el popup Tipo del
// gestor de coordenadas): si en cada redibujo se recalculara la
// posición contra el punto de click original, un crecimiento de
// tamaño puede cruzar el borde de la ventana y hacer que el
// popup "salte" de lugar (cambio de anclaje izq/der o arriba/
// abajo). Acá se preserva el anclaje ya elegido la primera vez.
// ======================================================

export function actualizarContenidoPopup(contenido: HTMLElement): void {
  if (!capaPopup) {
    return;
  }

  const anterior = capaPopup.firstElementChild as HTMLElement | null;

  const posicion = anterior
    ? {
        position: anterior.style.position,
        left: anterior.style.left,
        top: anterior.style.top,
        right: anterior.style.right,
        bottom: anterior.style.bottom,
      }
    : null;

  capaPopup.innerHTML = "";
  capaPopup.append(contenido);

  if (posicion) {
    contenido.style.position = posicion.position;
    contenido.style.left = posicion.left;
    contenido.style.top = posicion.top;
    contenido.style.right = posicion.right;
    contenido.style.bottom = posicion.bottom;
  }
}

export function ocultarPopup(): void {
  if (!capaPopup) {
    return;
  }

  capaPopup.style.display = "none";

  popupFijoActual = false;

  if (origenActual) {
    origenActual.dataset.abierto = "false";
    origenActual = null;
  }

  const alCerrar = alCerrarActual;

  alCerrarActual = null;

  alCerrar?.();
}
