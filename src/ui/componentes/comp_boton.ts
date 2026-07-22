// ======================================================
// 🔘 comp_Boton RemapH V3
// ======================================================

export interface BotonOpciones {
  texto: string;

  html?: string;

  clase?: string;

  titulo?: string;
}

export function crearBoton(opciones: BotonOpciones): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn";

  if (opciones.clase) {
    boton.classList.add(opciones.clase);
  }

  if (opciones.html) {
    boton.innerHTML = opciones.html;
  } else {
    boton.textContent = opciones.texto;
  }

  boton.title = opciones.titulo ?? opciones.texto;

  return boton;
}
