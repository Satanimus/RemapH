// ======================================================
// 🔧 comp_Popup
// ------------------------------------------------------
// La columna Extra (único consumidor de este helper) ya no
// muestra texto: es solo el ícono 🔧, mismo criterio icon-only
// que la columna Tipo (ver comp_popup_abrir.ts, iconoDeTipo).
// Las opciones elegidas quedan como tooltip (title), con formato
// "Subtítulo: Elección" por línea — cada texto*Extra()/textoMenuExtra()/
// etc. arma ese contenido.
// ======================================================

import { crearBoton } from "./comp_boton";

export interface PopupOpciones {
  titulo: string;
  onClick?: (evento: MouseEvent) => void;
}

export function crearPopup(opciones: PopupOpciones): HTMLButtonElement {
  const boton = crearBoton({
    texto: "🔧",
    titulo: opciones.titulo,
    clase: "extra-control",
  });

  if (opciones.onClick) {
    boton.addEventListener("click", opciones.onClick);
  }

  return boton;
}
