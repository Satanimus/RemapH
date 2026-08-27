// ======================================================
// 🪟 comp_Popup_Separadores
// ------------------------------------------------------
// Popup de opciones del header de Separadores (botón "⁝"
// de ui_separador.ts). Calcado de abrirPopupOpciones() en
// comp_popup_abrir.ts (mismo patrón dibujar()/mostrarPopup/
// ocultarPopup), con las diferencias puntuales del separador:
// Color escribe separador.color y repinta con reconstruirTabla()
// (no reconstruirFila, que no sirve para un id de separador);
// Clonar es un solo botón sin la lógica de filaTieneAccion;
// Mover queda listado sin función real (Etapa D); Eliminar
// NO usa doble confirmación — expande una caja con 3 botones
// que ejecutan directo en su propio clic, como pide el
// resumen general.
// ======================================================

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import type { SeparadorPerfil } from "../core/core_perfil";

import {
  clonarSeparadoresPorId,
  eliminarSeparadoresConFilas,
  moverSeparadoresFuera,
} from "../core/core_perfil_acciones";

import { COLOR_OPCIONES } from "./comp_popup_abrir";

import { reconstruirTabla, activarModoMoverTabla } from "../ui/ui_tabla_control";

// ======================================================
// ➖ SEPARADOR (mismo estilo que el resto de los popups)
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

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
// ⁝ POPUP OPCIONES DE SEPARADOR
// ======================================================

export function abrirPopupSeparadores(
  evento: MouseEvent,
  separador: SeparadorPerfil,
  alModificar: () => void,
): void {
  let colorExpandido = false;

  let eliminarExpandido = false;

  const dibujar = (): void => {
    const lista = document.createElement("div");

    lista.className = "popup-lista";

    lista.dataset.ayudaId = "popup-opciones-fila";

    // ----------------------------------
    // 🎨 COLOR (se expande en una caja interna)
    // ----------------------------------

    const botonColor = document.createElement("button");

    botonColor.className = "ui-btn";

    if (separador.color) {
      botonColor.classList.add("popup-color-item");

      const opcion = COLOR_OPCIONES.find((o) => o.valor === separador.color);

      const muestra = document.createElement("span");

      muestra.className = "popup-color-muestra";
      muestra.style.background = `var(--tag-${separador.color})`;

      const texto = document.createElement("span");

      texto.textContent = `Color ${opcion?.texto ?? separador.color}`;

      botonColor.append(muestra, texto);
    } else {
      botonColor.textContent = "🎨 Color";
    }

    botonColor.addEventListener("click", () => {
      colorExpandido = !colorExpandido;
      dibujar();
    });

    lista.append(botonColor);

    if (colorExpandido) {
      const caja = document.createElement("div");

      caja.className = "popup-caja-interna";

      llenarListaColorSeparador(caja, separador, () => {
        alModificar();
        ocultarPopup();
      });

      lista.append(caja);
    }

    lista.append(crearSeparador());

    // ----------------------------------
    // 📋 CLONAR
    // ----------------------------------

    const botonClonar = document.createElement("button");

    botonClonar.className = "ui-btn";
    botonClonar.textContent = "Clonar";

    botonClonar.addEventListener("click", () => {
      clonarSeparadoresPorId(separador.id);
      alModificar();
      reconstruirTabla();
      ocultarPopup();
    });

    // ----------------------------------
    // ⁝⁝ MOVER
    // ------------------------------------------------------
    // La función real llega en la Etapa D.
    // ----------------------------------

    const botonMover = document.createElement("button");

    botonMover.className = "ui-btn";
    botonMover.textContent = "Mover";

    botonMover.addEventListener("click", () => {
      activarModoMoverTabla(separador.id);
      ocultarPopup();
    });

    lista.append(botonClonar, botonMover);

    // ----------------------------------
    // 🗑️ ELIMINAR (sin doble confirmación: se expande una
    // caja con 3 botones, cada uno ejecuta directo)
    // ----------------------------------

    const botonEliminar = document.createElement("button");

    botonEliminar.className = "ui-btn popup-perfil-eliminar";
    botonEliminar.textContent = "Eliminar";

    botonEliminar.addEventListener("click", () => {
      eliminarExpandido = !eliminarExpandido;
      dibujar();
    });

    lista.append(botonEliminar);

    if (eliminarExpandido) {
      const caja = document.createElement("div");

      caja.className = "popup-caja-interna";

      const botonEliminarFilas = document.createElement("button");

      botonEliminarFilas.className = "ui-btn popup-perfil-eliminar";
      botonEliminarFilas.textContent = "Eliminar filas";

      botonEliminarFilas.addEventListener("click", () => {
        eliminarSeparadoresConFilas(separador.id);
        alModificar();
        reconstruirTabla();
        ocultarPopup();
      });

      const botonMoverFuera = document.createElement("button");

      botonMoverFuera.className = "ui-btn";
      botonMoverFuera.textContent = "Mover fuera";

      botonMoverFuera.addEventListener("click", () => {
        moverSeparadoresFuera(separador.id);
        alModificar();
        reconstruirTabla();
        ocultarPopup();
      });

      const botonCancelar = document.createElement("button");

      botonCancelar.className = "ui-btn";
      botonCancelar.textContent = "Cancelar";

      botonCancelar.addEventListener("click", () => {
        eliminarExpandido = false;
        dibujar();
      });

      caja.append(botonEliminarFilas, botonMoverFuera, botonCancelar);

      lista.append(caja);
    }

    mostrarPopup(lista, evento.clientX, evento.clientY);
  };

  dibujar();
}
