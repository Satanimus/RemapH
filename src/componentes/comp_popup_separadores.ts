// ======================================================
// 🪟 comp_Popup_Separadores
// ------------------------------------------------------
// Popup de color del separador (botón 🎨 de la columna
// Opciones inline — ver ui_separador.ts / comp_opciones.ts).
// ======================================================

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import type { SeparadorPerfil } from "../core/core_perfil";

import { COLOR_OPCIONES } from "./comp_popup_abrir";

import { reconstruirTabla } from "../ui/ui_tabla_control";

// ======================================================
// 🎨 LISTA DE COLOR DEL SEPARADOR
// ------------------------------------------------------
// Misma UI que llenarListaColor() de la fila (comp_popup_abrir.ts),
// pero escribe separador.color y repinta con reconstruirTabla() —
// el color también repinta las filas contenidas (ver C7/C8).
// ======================================================

function llenarListaColorSeparador(
  contenedor: HTMLElement,
  separador: SeparadorPerfil,
  alSeleccionar: () => void,
): void {
  contenedor.replaceChildren();

  const botonLimpiar = document.createElement("button");

  botonLimpiar.className = "ui-btn";
  botonLimpiar.textContent = "🎨 Limpiar";

  botonLimpiar.addEventListener("click", () => {
    separador.color = "";

    reconstruirTabla();
    alSeleccionar();
  });

  contenedor.append(botonLimpiar);

  COLOR_OPCIONES.forEach((opcion) => {
    const boton = document.createElement("button");

    boton.className = "ui-btn popup-color-item";

    const muestra = document.createElement("span");

    muestra.className = "popup-color-muestra";
    muestra.style.background = `var(--tag-${opcion.valor})`;

    const texto = document.createElement("span");

    texto.textContent = opcion.texto;

    boton.append(muestra, texto);

    boton.addEventListener("click", () => {
      separador.color = opcion.valor;

      reconstruirTabla();
      alSeleccionar();
    });

    contenedor.append(boton);
  });
}

// ======================================================
// 🎨 POPUP DE COLOR — SEPARADOR
// ------------------------------------------------------
// Reutiliza llenarListaColorSeparador (misma paleta y misma
// escritura en separador.color) para el botón 🎨 de la
// columna Opciones del separador — ver ui_separador.ts.
// ======================================================

export function abrirPopupColorSeparador(
  evento: MouseEvent,
  separador: SeparadorPerfil,
): void {
  const lista = document.createElement("div");

  lista.className = "popup-lista";

  llenarListaColorSeparador(lista, separador, ocultarPopup);

  mostrarPopup(lista, evento.clientX, evento.clientY);
}


