// ======================================================
// 🪟 comp_Popup_Abrir
// ======================================================

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import type { ContextoFila } from "../core/core_contexto_fila";
import {
  clonarFilaPorId,
  eliminarFilaPorId,
  filaTieneAccion,
} from "../core/core_perfil_acciones";
import type { FilaPerfil } from "../core/core_perfil";
import { crearEntrada } from "../core/core_entrada";
import { crearTrigger } from "../core/core_trigger";
import { reconstruirFila } from "../ui/ui_tabla_control";
import { reconstruirTabla } from "../ui/ui_tabla_control";
import { activarModoMoverTabla } from "../ui/ui_tabla_control";

// ======================================================
// ➖ SEPARADOR (mismo estilo que el resto de los popups)
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

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
// reales (los que espera Rust — ver compilador.rs). El
// ícono de cada tipo es el mismo que ya usa por defecto la
// columna Acción para ese tipo (ver textoAccionMultimedia
// en core_multimedia.ts, textoMenuAccion en
// core_menu_express.ts, textoMacroAccion en core_macro.ts,
// textoPortapapelesAccion en core_portapapeles.ts, y el
// ícono de carpeta de comp_popup_abrir_accion.ts) — acá se
// repite como texto plano porque este popup no depende de
// ninguno de esos módulos.
// ======================================================

const TIPO_OPCIONES: { texto: string; icono: string; valor: string }[] = [
  { texto: "Teclado - Mouse", icono: "🔠", valor: "tecla_mouse" },
  { texto: "Multimedia", icono: "🎵", valor: "multimedia" },
  { texto: "MenuExpress", icono: "⚡", valor: "menu_express" },
  { texto: "Macro", icono: "🧩", valor: "macro" },
  { texto: "Portapapeles", icono: "📋", valor: "portapapeles" },
  { texto: "Abrir Archivo/App", icono: "📂", valor: "abrir" },
];

const EXTRA_OPCIONES: { texto: string; valor: string }[] = [
  { texto: "Normal", valor: "" },
  { texto: "Turbo", valor: "turbo" },
  { texto: "Mantener", valor: "mantener" },
];

export function tipoATexto(valor: string): string {
  const opcion = TIPO_OPCIONES.find((opcion) => opcion.valor === valor);

  return opcion ? `${opcion.texto}` : valor;
}

export function iconoDeTipo(valor: string): string {
  return TIPO_OPCIONES.find((opcion) => opcion.valor === valor)?.icono ?? "";
}

export function extraATexto(valor: string): string {
  const opcion = EXTRA_OPCIONES.find((opcion) => opcion.valor === valor);

  return `Repetición: ${opcion ? opcion.texto : valor}`;
}

export function abrirPopupCondicion(
  evento: MouseEvent,
  actualizar: (texto: string) => void,
): void {
  abrirLista(evento, ["Normal", "Mantener pulsado", "Doble toque"], actualizar);
}

// ======================================================
// 🔤🎵⚡🧩📋📂 POPUP TIPO — lista con ícono al lado del nombre
// ------------------------------------------------------
// Variante de crearListaConValor() que antepone el ícono del
// tipo (TIPO_OPCIONES.icono) a cada botón — el resto de las
// listas con valor (abrirListaConValor, usada por Extra)
// sigue solo con texto, sin tocar.
// ======================================================

function crearListaTipo(
  opciones: { texto: string; icono: string; valor: string }[],
  seleccion?: (valor: string) => void,
): HTMLElement {
  const lista = document.createElement("div");

  lista.className = "popup-lista";

  opciones.forEach((opcion) => {
    const boton = document.createElement("button");

    boton.className = "ui-btn popup-tipo-item";

    const icono = document.createElement("span");

    icono.className = "popup-tipo-icono";
    icono.textContent = opcion.icono;

    const texto = document.createElement("span");

    texto.textContent = opcion.texto;

    boton.append(icono, texto);

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

export function abrirPopupTipo(
  evento: MouseEvent,
  actualizar: (valor: string) => void,
  _contexto: ContextoFila,
): void {
  mostrarPopup(
    crearListaTipo(TIPO_OPCIONES, actualizar),
    evento.clientX,
    evento.clientY,
  );
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

export const COLOR_OPCIONES: { texto: string; valor: string }[] = [
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
// abre un popup nuevo: expande una caja interna (misma clase
// popup-caja-interna que usa, por ejemplo, "Relativa a
// Ventana" dentro del popup Extra de Coordenada) justo debajo
// del botón, dentro de ESTE MISMO popup — nunca crea un
// elemento de popup aparte, así conserva el position:fixed +
// left/top que mostrarPopup ya le puso inline según el click
// original, y el popup no salta de lugar. dibujar() se llama
// a sí misma en cada redibujado para que colorExpandido
// sobreviva mientras el popup sigue abierto (mismo patrón que
// abrirConExpandido en comp_popup_abrir_extra.ts). "Mover"
// activa el modo mover del componente de arrastre
// (util_arrastrable.ts).
// ======================================================

export function abrirPopupOpciones(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  let colorExpandido = false;

  const dibujar = (): void => {
    const lista = document.createElement("div");

    lista.className = "popup-lista";

    // ----------------------------------
    // 🎨 COLOR (se expande en una caja interna)
    // ----------------------------------

    const botonColor = document.createElement("button");

    botonColor.className = "ui-btn";

    if (filaPerfil.color) {
      botonColor.classList.add("popup-color-item");

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
      colorExpandido = !colorExpandido;
      dibujar();
    });

    lista.append(botonColor);

    if (colorExpandido) {
      const caja = document.createElement("div");

      caja.className = "popup-caja-interna";

      llenarListaColor(caja, contexto, filaPerfil, () => {
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

    lista.append(botonClonar, botonMover, botonEliminar);

    mostrarPopup(lista, evento.clientX, evento.clientY);
  };

  dibujar();
}

export function abrirPopupExtra(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  abrirListaConValor(evento, EXTRA_OPCIONES, (valor) => {
    filaPerfil.extra = valor;

    reconstruirFila(contexto.id);

    alModificar();
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
  // ➕ ESC +
  // ----------------------------------

  const botonWin = document.createElement("button");

  botonWin.className = "ui-btn";
  botonWin.textContent = "Esc +";

  botonWin.addEventListener("click", () => {
    const entrada = crearModificador("Esc +");

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
    case "Esc +":
      return crearEntrada("Teclado", "Escape", "Esc");

    default:
      return null;
  }
}
