// ======================================================
// 🪟 comp_Popup_Abrir
// ======================================================

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import type { ContextoFila } from "../../core/core_contexto_fila";
import {
  clonarFilaPorId,
  eliminarFilaPorId,
  filaTieneAccion,
} from "../../core/core_perfil_acciones";
import type { FilaPerfil } from "../../core/core_perfil";
import { crearEntrada } from "../../core/core_entrada";
import { crearTrigger } from "../../core/core_trigger";
import { reconstruirFila } from "../ui_tabla_control";
import { reconstruirTabla } from "../ui_tabla_control";
import { activarModoMoverTabla } from "../ui_tabla_control";

function crearLista(
  opciones: string[],
  seleccion?: (valor: string) => void,
): HTMLElement {
  const lista = document.createElement("div");

  lista.className = "popup-lista";

  opciones.forEach((opcion) => {
    const boton = document.createElement("button");

    boton.className = "ui-btn";

    boton.textContent = opcion;

    boton.addEventListener("click", () => {
      if (seleccion) {
        seleccion(opcion);
      }

      ocultarPopup();
    });

    lista.append(boton);
  });

  return lista;
}

function abrirLista(
  evento: MouseEvent,
  opciones: string[],
  actualizar: (texto: string) => void,
): void {
  mostrarPopup(
    crearLista(opciones, actualizar),
    evento.clientX,
    evento.clientY,
  );
}

// ======================================================
// 🏷️ LISTA CON VALOR SEPARADO DEL TEXTO
// ------------------------------------------------------
// Para popups donde lo que se muestra (label, con mayúscula,
// legible) no es lo que se guarda (valor, en minúscula — el
// mismo formato que espera Rust).
// ======================================================

function crearListaConValor(
  opciones: { texto: string; valor: string }[],
  seleccion?: (valor: string) => void,
): HTMLElement {
  const lista = document.createElement("div");

  lista.className = "popup-lista";

  opciones.forEach((opcion) => {
    const boton = document.createElement("button");

    boton.className = "ui-btn";

    boton.textContent = opcion.texto;

    boton.addEventListener("click", () => {
      if (seleccion) {
        seleccion(opcion.valor);
      }

      ocultarPopup();
    });

    lista.append(boton);
  });

  return lista;
}

function abrirListaConValor(
  evento: MouseEvent,
  opciones: { texto: string; valor: string }[],
  actualizar: (valor: string) => void,
): void {
  mostrarPopup(
    crearListaConValor(opciones, actualizar),
    evento.clientX,
    evento.clientY,
  );
}

// ======================================================
// 🧭 VOCABULARIO tipo / extra
// ------------------------------------------------------
// Única fuente de verdad para las opciones y sus valores
// reales (los que espera Rust — ver compilador.rs).
// ======================================================

const TIPO_OPCIONES: { texto: string; valor: string }[] = [
  { texto: "Tecla/Mouse", valor: "tecla_mouse" },
  { texto: "Multimedia", valor: "multimedia" },
  { texto: "MenuExpress", valor: "menu_express" },
  { texto: "Macro", valor: "macro" },
  { texto: "Portapapeles", valor: "portapapeles" },
  { texto: "Abrir Archivo/App", valor: "abrir" },
];

const EXTRA_OPCIONES: { texto: string; valor: string }[] = [
  { texto: "Normal", valor: "" },
  { texto: "Turbo", valor: "turbo" },
  { texto: "Mantener", valor: "mantener" },
];

export function tipoATexto(valor: string): string {
  return TIPO_OPCIONES.find((opcion) => opcion.valor === valor)?.texto ?? valor;
}

export function extraATexto(valor: string): string {
  return (
    EXTRA_OPCIONES.find((opcion) => opcion.valor === valor)?.texto ?? valor
  );
}

export function abrirPopupCondicion(
  evento: MouseEvent,
  actualizar: (texto: string) => void,
): void {
  abrirLista(evento, ["Normal", "Mantener pulsado", "Doble toque"], actualizar);
}

export function abrirPopupTipo(
  evento: MouseEvent,
  actualizar: (valor: string) => void,
  _contexto: ContextoFila,
): void {
  abrirListaConValor(evento, TIPO_OPCIONES, actualizar);
}

export function abrirPopupEstado(
  evento: MouseEvent,
  actualizar: (texto: string) => void,
  contexto: ContextoFila,
): void {
  abrirLista(evento, ["ON", "OFF", "Clonar", "Eliminar"], (texto) => {
    if (texto === "ON" || texto === "OFF") {
      actualizar(texto);
    }

    if (texto === "Clonar") {
      clonarFilaPorId(contexto.id);

      reconstruirTabla();
    }
  });
}

// ======================================================
// 🔢 POPUP NÚMERO DE FILA
// ------------------------------------------------------
// Clonar / Eliminar. "Mover" no es una opción de este menú
// — se activa con clic MANTENIDO sobre el mismo botón,
// resuelto enteramente por util_arrastrable.ts (ver
// registrarFila en ui_tabla.ts). Acá solo se atiende el
// clic CORTO. No usa abrirLista porque Eliminar necesita
// doble confirmación in-place, y ambas acciones modifican
// el perfil (marcan editado).
// ======================================================

export function abrirPopupNumero(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  const lista = document.createElement("div");

  lista.className = "popup-lista";

  // ----------------------------------
  // 📋 CLONAR
  // ----------------------------------

  const botonClonar = document.createElement("button");

  botonClonar.className = "ui-btn";
  botonClonar.textContent = "Clonar";

  botonClonar.addEventListener("click", () => {
    clonarFilaPorId(contexto.id);
    alModificar();
    reconstruirTabla();
    ocultarPopup();
  });

  // ----------------------------------
  // 🗑️ ELIMINAR
  // ----------------------------------

  const botonEliminar = document.createElement("button");

  botonEliminar.className = "ui-btn popup-perfil-eliminar";
  botonEliminar.textContent = "Eliminar";

  let confirmando = false;

  botonEliminar.addEventListener("click", () => {
    if (filaTieneAccion(filaPerfil) && !confirmando) {
      confirmando = true;

      botonEliminar.textContent = "⚠️ Confirmar eliminación";

      return;
    }

    eliminarFilaPorId(contexto.id);
    alModificar();
    reconstruirTabla();
    ocultarPopup();
  });

  lista.append(botonClonar, botonEliminar);

  mostrarPopup(lista, evento.clientX, evento.clientY);
}

// ======================================================
// 🎨 PALETA DE COLORES DE FILA (lista reutilizable)
// ------------------------------------------------------
// El valor guardado (`valor`) es la clave usada para armar
// la variable CSS --tag-<valor> (ver styl_variables.css).
// No es un color literal: así el estilo queda editable
// desde un solo lugar sin tocar este archivo. Se usa tanto
// en el popup de Color "suelto" (abrirPopupColor, hoy sin
// uso desde que la columna Color se sacó de la tabla) como
// dentro del popup de Opciones (abrirPopupOpciones), donde
// se muestra expandida en el mismo popup en vez de abrir
// uno nuevo.
// ======================================================

const COLOR_OPCIONES: { texto: string; valor: string }[] = [
  { texto: "Rojo", valor: "red" },
  { texto: "Naranja", valor: "orange" },
  { texto: "Amarillo", valor: "yellow" },
  { texto: "Verde", valor: "green" },
  { texto: "Cian", valor: "cyan" },
  { texto: "Azul", valor: "blue" },
  { texto: "Morado", valor: "purple" },
  { texto: "Rosa", valor: "pink" },
  { texto: "Gris", valor: "gray" },
];

function llenarListaColor(
  contenedor: HTMLElement,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alSeleccionar: () => void,
): void {
  contenedor.replaceChildren();

  // ----------------------------------
  // 🎨 LIMPIAR
  // ----------------------------------

  const botonLimpiar = document.createElement("button");

  botonLimpiar.className = "ui-btn";
  botonLimpiar.textContent = "🎨 Limpiar";

  botonLimpiar.addEventListener("click", () => {
    filaPerfil.color = "";

    reconstruirFila(contexto.id);
    alSeleccionar();
  });

  contenedor.append(botonLimpiar);

  // ----------------------------------
  // 🎨 OPCIONES DE COLOR
  // ----------------------------------

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
      filaPerfil.color = opcion.valor;

      reconstruirFila(contexto.id);
      alSeleccionar();
    });

    contenedor.append(boton);
  });
}

export function abrirPopupColor(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const lista = document.createElement("div");

  lista.className = "popup-lista";

  llenarListaColor(lista, contexto, filaPerfil, ocultarPopup);

  mostrarPopup(lista, evento.clientX, evento.clientY);
}

// ======================================================
// ⁝ POPUP OPCIONES DE FILA
// ------------------------------------------------------
// Reemplaza a abrirPopupNumero como menú principal de la
// fila (ver comp_opciones.ts, Etapa D del plan). "Color" no
// abre un popup nuevo: vacía y vuelve a llenar este MISMO
// div "lista" con la paleta (llenarListaColor) en vez de
// crear un elemento nuevo — así conserva el position:fixed
// + left/top que mostrarPopup ya le puso inline según el
// click original, y el popup no salta de lugar. "Mover"
// activa el modo mover del componente de arrastre
// (util_arrastrable.ts) — queda pendiente hasta la Etapa E
// del plan; por ahora solo cierra el popup.
// ======================================================

export function abrirPopupOpciones(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  const lista = document.createElement("div");

  lista.className = "popup-lista";

  // ----------------------------------
  // 🎨 COLOR (se expande en el mismo popup)
  // ----------------------------------

  const botonColor = document.createElement("button");

  botonColor.className = "ui-btn";

  if (filaPerfil.color) {
    const opcion = COLOR_OPCIONES.find((o) => o.valor === filaPerfil.color);

    const muestra = document.createElement("span");

    muestra.className = "popup-color-muestra";
    muestra.style.background = `var(--tag-${filaPerfil.color})`;

    const texto = document.createElement("span");

    texto.textContent = `Color ${opcion?.texto ?? filaPerfil.color}`;

    botonColor.append(muestra, texto);
  } else {
    botonColor.textContent = "🎨 Color";
  }

  botonColor.addEventListener("click", () => {
    llenarListaColor(lista, contexto, filaPerfil, () => {
      alModificar();
      ocultarPopup();
    });
  });

  // ----------------------------------
  // 📋 CLONAR
  // ----------------------------------

  const botonClonar = document.createElement("button");

  botonClonar.className = "ui-btn";
  botonClonar.textContent = "Clonar";

  botonClonar.addEventListener("click", () => {
    clonarFilaPorId(contexto.id);
    alModificar();
    reconstruirTabla();
    ocultarPopup();
  });

  // ----------------------------------
  // ⁝⁝ MOVER
  // ------------------------------------------------------
  // Activa el modo mover del controlador de arrastre para
  // esta fila (ver activarModoMoverTabla en ui_tabla_control.ts
  // → registrarActivarModoMover en ui_tabla.ts → activarModoMoverPara
  // en util_arrastrable.ts). A partir de acá el usuario arrastra
  // con el mouse o mueve con las flechas, igual que con el
  // clic mantenido sobre el asa.
  // ----------------------------------

  const botonMover = document.createElement("button");

  botonMover.className = "ui-btn";
  botonMover.textContent = "Mover";

  botonMover.addEventListener("click", () => {
    activarModoMoverTabla(contexto.id);
    ocultarPopup();
  });

  // ----------------------------------
  // 🗑️ ELIMINAR
  // ----------------------------------

  const botonEliminar = document.createElement("button");

  botonEliminar.className = "ui-btn popup-perfil-eliminar";
  botonEliminar.textContent = "Eliminar";

  let confirmando = false;

  botonEliminar.addEventListener("click", () => {
    if (filaTieneAccion(filaPerfil) && !confirmando) {
      confirmando = true;

      botonEliminar.textContent = "⚠️ Confirmar eliminación";

      return;
    }

    eliminarFilaPorId(contexto.id);
    alModificar();
    reconstruirTabla();
    ocultarPopup();
  });

  lista.append(botonColor, botonClonar, botonMover, botonEliminar);

  mostrarPopup(lista, evento.clientX, evento.clientY);
}

export function abrirPopupExtra(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  abrirListaConValor(evento, EXTRA_OPCIONES, (valor) => {
    filaPerfil.extra = valor;

    reconstruirFila(contexto.id);
  });
}

// ======================================================
// ➕ POPUP MODIFICADOR (botón "+" del capturador)
// ------------------------------------------------------
// No usa abrirLista() porque "Eliminar Captura" necesita su propio
// estilo (rojo, ver .popup-perfil-eliminar) distinto del resto de
// las opciones — mismo motivo que abrirPopupNumero más abajo.
// ======================================================

export function abrirPopupModificador(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  destino: "Trigger" | "Accion" = "Trigger",
): void {
  const lista = document.createElement("div");

  lista.className = "popup-lista";

  // ----------------------------------
  // ➕ WIN +
  // ----------------------------------

  const botonWin = document.createElement("button");

  botonWin.className = "ui-btn";
  botonWin.textContent = "Win +";

  botonWin.addEventListener("click", () => {
    const entrada = crearModificador("Win +");

    const trigger =
      destino === "Trigger" ? filaPerfil.trigger : filaPerfil.accion;

    if (entrada && trigger) {
      const existe = trigger.modificadores.some(
        (modificador) => modificador.codigo === entrada.codigo,
      );

      if (!existe) {
        trigger.modificadores.unshift(entrada);

        reconstruirFila(contexto.id);
      }
    }

    ocultarPopup();
  });

  // ----------------------------------
  // 🗑️ ELIMINAR CAPTURA
  // ------------------------------------------------
  // Borra el Trigger/Acción ya capturado en esta fila y el botón
  // grande vuelve a "🚩 Capturar". Simple, sin doble confirmación
  // (a diferencia de "Eliminar" fila/perfil) — acá se pierde solo
  // esta captura puntual, no toda la fila.
  // ------------------------------------------------

  const botonEliminar = document.createElement("button");

  botonEliminar.className = "ui-btn popup-perfil-eliminar";
  botonEliminar.textContent = "Eliminar Captura";

  botonEliminar.addEventListener("click", () => {
    if (destino === "Trigger") {
      filaPerfil.trigger = crearTrigger();
    } else {
      filaPerfil.accion = null;
    }

    reconstruirFila(contexto.id);

    ocultarPopup();
  });

  lista.append(botonWin, botonEliminar);

  mostrarPopup(lista, evento.clientX, evento.clientY);
}

function crearModificador(texto: string) {
  switch (texto) {
    case "Win +":
      return crearEntrada("Teclado", "MetaLeft", "Win");

    default:
      return null;
  }
}
