// ======================================================
// ⚡📝 comp_Popup_Menu_Express_Editor
// ------------------------------------------------------
// Editor del MenuExpress (filaPerfil.tipo === "menu_express"),
// abierto al hacer clic en la columna Acción (ver comp_accion.ts).
// Mismo patrón persistente que el resto de popups Extra: cada
// interacción actualiza filaPerfil.menuAccion y redibuja el mismo
// popup en el lugar. No hay botón guardar/cancelar — todo se guarda
// al toque (mismo criterio que elegir Forma/Comportamiento en el
// popup Extra) y se cierra solo al hacer clic fuera (mostrarPopup ya
// resuelve eso, ver comp_popup_contenedor.ts).
//
// DISPONIBLES: todas las filas del perfil actual, excepto
//   • filas con tipo === "menu_express" (incluida esta misma fila —
//     evita anidar menús, ver spec)
//   • filas ya agregadas a este menú
// SELECCIONADAS: las filas en menuAccion.botones, ordenadas por
//   número de fila (posición en perfil.filas, no por orden de
//   agregado) — si alguna fue eliminada del perfil mientras el
//   editor estaba abierto, se omite acá (compilador.rs hace el
//   mismo descarte silencioso al compilar).
//
// El número mostrado (#N) es la posición de la fila en la tabla
// (1-based, la misma que ve el usuario en la columna Número) — NO
// es filaId, que es el identificador interno usado para guardar
// (ver core_menu_express.ts).
// ======================================================

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui_tabla_control";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import { obtenerPerfilUi } from "../../core/core_perfil_ui";

import { textoAccionFila } from "../../core/core_perfil_acciones";

import type { MenuBotonPerfil } from "../../core/core_menu_express";

import { esSeparador } from "../../core/core_agrupacion";

// ======================================================
// 🎨 INDICADOR DE COLOR DE FILA
// ------------------------------------------------------
// Mismo círculo que usa la paleta de color de fila (ver
// comp_popup_abrir.ts::abrirPopupColor) — vacío/gris si la fila no
// tiene color asignado.
// ======================================================

function crearIndicadorColor(color: string): HTMLElement {
  const muestra = document.createElement("span");

  muestra.className = "popup-color-muestra";

  muestra.style.background = color
    ? `var(--tag-${color})`
    : "var(--border-light)";

  return muestra;
}

// ======================================================
// 🔢 NÚMERO DE FILA (posición en la tabla, 1-based)
// ======================================================

function numeroDeFila(fila: FilaPerfil, todasLasFilas: FilaPerfil[]): number {
  return todasLasFilas.indexOf(fila) + 1;
}

// ======================================================
// 📄 ITEM DISPONIBLE ([+] agregar)
// ======================================================

function crearItemDisponible(
  fila: FilaPerfil,
  numero: number,
  onAgregar: () => void,
): HTMLElement {
  const item = document.createElement("div");

  item.className = "popup-menu-editor-item";
  item.dataset.seccion = "disponible";

  const botonAgregar = document.createElement("button");

  botonAgregar.className =
    "ui-btn popup-menu-editor-toggle popup-menu-editor-agregar";
  botonAgregar.textContent = "+";
  botonAgregar.title = "Agregar al menú";

  botonAgregar.addEventListener("click", onAgregar);

  const numeroSpan = document.createElement("span");

  numeroSpan.className = "popup-menu-editor-numero";
  numeroSpan.textContent = `#${numero}`;

  const texto = document.createElement("span");

  texto.className = "popup-menu-editor-texto";
  texto.textContent = textoAccionFila(fila);

  item.append(botonAgregar, crearIndicadorColor(fila.color), numeroSpan, texto);

  return item;
}

// ======================================================
// 📄 ITEM SELECCIONADO ([x] quitar + Renombrar)
// ======================================================

function crearItemSeleccionado(
  fila: FilaPerfil,
  numero: number,
  renombrar: string,
  onQuitar: () => void,
  onRenombrar: (valor: string) => void,
): HTMLElement {
  const item = document.createElement("div");

  item.className = "popup-menu-editor-item";
  item.dataset.seccion = "seleccionado";

  const botonQuitar = document.createElement("button");

  botonQuitar.className =
    "ui-btn popup-menu-editor-toggle popup-menu-editor-quitar";
  botonQuitar.textContent = "×";
  botonQuitar.title = "Quitar del menú";

  botonQuitar.addEventListener("click", onQuitar);

  const numeroSpan = document.createElement("span");

  numeroSpan.className = "popup-menu-editor-numero";
  numeroSpan.textContent = `#${numero}`;

  const inputRenombrar = document.createElement("input");

  inputRenombrar.type = "text";
  inputRenombrar.className = "popup-input popup-menu-editor-renombrar";
  inputRenombrar.value = renombrar;
  inputRenombrar.title = textoAccionFila(fila);

  inputRenombrar.addEventListener("input", () => {
    onRenombrar(inputRenombrar.value);
  });

  item.append(
    botonQuitar,
    crearIndicadorColor(fila.color),
    numeroSpan,
    inputRenombrar,
  );

  return item;
}

// ======================================================
// ⚡📝 ABRIR EDITOR MENUEXPRESS
// ======================================================

export function abrirEditorMenuExpress(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const todasLasFilas = obtenerPerfilUi().filas.filter(
    (item): item is FilaPerfil => !esSeparador(item),
  );

  const menuAccion = filaPerfil.menuAccion;

  const redibujar = () => abrirEditorMenuExpress(evento, contexto, filaPerfil);

  const guardarYRedibujar = () => {
    reconstruirFila(contexto.id);
    redibujar();
  };

  const popup = document.createElement("div");

  popup.className = "popup-extra popup-menu-editor";

  // ----------------------------------
  // ✏️ NOMBRE DEL MENÚ
  // ----------------------------------

  const inputNombre = document.createElement("input");

  inputNombre.type = "text";
  inputNombre.className = "popup-input popup-menu-editor-nombre";
  inputNombre.placeholder = "Nombre del menú";
  inputNombre.value = menuAccion.nombre;

  inputNombre.addEventListener("input", () => {
    menuAccion.nombre = inputNombre.value;

    // No se redibuja el popup entero acá (perdería el foco del
    // input mientras se escribe) — solo se refleja en la columna
    // Acción de la tabla, que sí puede reconstruirse en caliente.
    reconstruirFila(contexto.id);
  });

  popup.append(inputNombre);

  // ----------------------------------
  // ✅ BOTONES SELECCIONADOS
  // ----------------------------------

  const tituloSeleccionados = document.createElement("span");

  tituloSeleccionados.className = "popup-fila-label";
  tituloSeleccionados.textContent =
    "Botones Seleccionado - Nombre Personalizado";

  popup.append(tituloSeleccionados);

  const listaSeleccionados = document.createElement("div");

  listaSeleccionados.className =
    "popup-menu-editor-lista popup-menu-editor-lista--completa";

  const seleccionados = menuAccion.botones
    .map((boton) => ({
      boton,
      fila: todasLasFilas.find((fila) => fila.id === boton.filaId),
    }))
    // Fila borrada del perfil mientras el editor estaba abierto —
    // se omite (mismo criterio de descarte silencioso del compilador).
    .filter(
      (entrada): entrada is { boton: MenuBotonPerfil; fila: FilaPerfil } =>
        !!entrada.fila,
    )
    .sort(
      (a, b) => todasLasFilas.indexOf(a.fila) - todasLasFilas.indexOf(b.fila),
    );

  if (seleccionados.length === 0) {
    const vacio = document.createElement("span");

    vacio.className = "app-popup-lista-titulo";
    vacio.textContent = "Todavía no agregaste ningún botón";

    listaSeleccionados.append(vacio);
  }

  seleccionados.forEach(({ boton, fila }) => {
    listaSeleccionados.append(
      crearItemSeleccionado(
        fila,
        numeroDeFila(fila, todasLasFilas),
        boton.renombrar,
        () => {
          menuAccion.botones = menuAccion.botones.filter(
            (item) => item.filaId !== fila.id,
          );

          guardarYRedibujar();
        },
        (valor) => {
          boton.renombrar = valor;

          // Acá tampoco se redibuja el popup completo — perdería
          // el foco del input de Renombrar mientras se escribe.
          reconstruirFila(contexto.id);
        },
      ),
    );
  });

  popup.append(listaSeleccionados);

  // ----------------------------------
  // ➕ BOTONES DISPONIBLES
  // ----------------------------------

  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  popup.append(separador);

  const tituloDisponibles = document.createElement("span");

  tituloDisponibles.className = "popup-fila-label";
  tituloDisponibles.textContent = "Botones Disponibles";

  popup.append(tituloDisponibles);

  const listaDisponibles = document.createElement("div");

  listaDisponibles.className = "popup-menu-editor-lista";

  const idsSeleccionados = new Set(
    menuAccion.botones.map((boton) => boton.filaId),
  );

  const disponibles = todasLasFilas.filter(
    (fila) => fila.tipo !== "menu_express" && !idsSeleccionados.has(fila.id),
  );

  if (disponibles.length === 0) {
    const vacio = document.createElement("span");

    vacio.className = "app-popup-lista-titulo";
    vacio.textContent = "No quedan filas disponibles para agregar";

    listaDisponibles.append(vacio);
  }

  disponibles.forEach((fila) => {
    listaDisponibles.append(
      crearItemDisponible(fila, numeroDeFila(fila, todasLasFilas), () => {
        menuAccion.botones.push({
          filaId: fila.id,

          // Se llena automáticamente con el texto de Acción actual
          // de esa fila — el usuario la puede renombrar después
          // desde la lista de Seleccionados.
          renombrar: textoAccionFila(fila),
        });

        guardarYRedibujar();
      }),
    );
  });

  popup.append(listaDisponibles);

  mostrarPopup(popup, evento.clientX, evento.clientY, () => {
    reconstruirFila(contexto.id);
  });

  inputNombre.focus();
}
