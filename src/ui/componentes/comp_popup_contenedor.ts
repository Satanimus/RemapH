// ======================================================
// 🪟 comp_Popup_Contenedor RemapH V3
// ------------------------------------------------------
// Capa compartida por TODOS los popups.
//
// Cierra solo con click en el fondo, nunca por burbujeo
// desde el contenido (inputs, botones, etc).
// ======================================================

let capaPopup: HTMLElement | null = null;
let alCerrarActual: (() => void) | null = null;

export function crearContenedorPopup(): HTMLElement {
  if (capaPopup) {
    return capaPopup;
  }

  capaPopup = document.createElement("div");

  capaPopup.className = "popup-capa";

  capaPopup.addEventListener("click", (evento) => {
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
  if (!capaPopup) {
    return;
  }

  capaPopup.innerHTML = "";

  capaPopup.append(contenido);

  capaPopup.style.display = "block";

  alCerrarActual = alCerrar ?? null;

  if (x !== undefined && y !== undefined) {
    contenido.style.position = "fixed";
    contenido.style.left = `${x}px`;
    contenido.style.top = `${y}px`;
  } else {
    contenido.style.position = "static";
  }
}

export function ocultarPopup(): void {
  if (!capaPopup) {
    return;
  }

  capaPopup.style.display = "none";

  const alCerrar = alCerrarActual;

  alCerrarActual = null;

  alCerrar?.();
}
