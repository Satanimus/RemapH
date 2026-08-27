// ======================================================
// 🧩 comp_Popup_Macro_Accion
// ------------------------------------------------------
// Menú de la columna Acción del tipo "Macro" (filaPerfil.tipo ===
// "macro"), conectado desde comp_accion.ts.
//
// Estado sin macro asignada: solo "Nueva" y "Abrir".
// "Nueva" crea directo (sin formulario de nombre previo) y abre
// el editor. "Abrir" despliega un box anidado (mismo patrón que
// el botón Color en opciones de fila / comp_popup_abrir.ts, clase
// popup-caja-interna) con el listado de macros guardadas.
//
// Estado con macro asignada: "Editar", "Nueva", "Abrir",
// "Eliminar" (letra roja, borde rojo al hover). Renombrar y
// Clonar ya no viven acá — Renombrar pasó al popup del editor
// (comp_popup_macro_editor.ts) y Clonar fue reemplazado por
// "Guardar como" dentro del editor.
//
// filaPerfil.accionReferencia sigue siendo el único dato que esta
// fila guarda sobre la macro elegida (mismo campo genérico que ya
// usa Multimedia) — nombre, no contenido.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

import { reconstruirFila } from "../ui/ui_tabla_control";

import { abrirEditorMacro } from "./comp_popup_macro_editor";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

// ======================================================
// 📦 RESULTADO DE macro_nueva
// ------------------------------------------------------
// Espejo de MacroArchivoJson (macro_json.rs) — acá solo
// importa `nombre`, el resto (pasos) lo consume el editor.
// ======================================================

interface MacroArchivoResultado {
  nombre: string;
}

// ======================================================
// 🚪 ABRIR POPUP (menú principal)
// ======================================================

export function abrirPopupMacroAccion(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  // Puramente visual — arranca siempre colapsado en cada apertura
  // nueva del popup (mismo criterio que abrirConExpandido en
  // comp_popup_abrir_extra.ts).
  let listaAbiertaExpandida = false;

  const dibujar = (): void => {
    const contenedor = document.createElement("div");

    contenedor.className = "popup-perfil";

    contenedor.dataset.ayudaId = "popup-macro-accion";

    contenedor.append(
      crearMenuPrincipal(
        filaPerfil,
        contexto,
        alModificar,
        () => {
          listaAbiertaExpandida = !listaAbiertaExpandida;
          dibujar();
        },
        listaAbiertaExpandida,
      ),
    );

    if (listaAbiertaExpandida) {
      const caja = document.createElement("div");

      caja.className = "popup-caja-interna";

      caja.append(
        crearListaMacros(filaPerfil, contexto, alModificar, () => {
          listaAbiertaExpandida = false;
          dibujar();
        }),
      );

      contenedor.append(caja);
    }

    mostrarPopup(contenedor, evento.clientX, evento.clientY);
  };

  dibujar();
}

// ======================================================
// 📋 MENÚ PRINCIPAL
// ------------------------------------------------------
// Sin macro: Nueva · Abrir.
// Con macro: Editar · Nueva · Abrir · Eliminar.
// ======================================================

function crearMenuPrincipal(
  filaPerfil: FilaPerfil,
  contexto: ContextoFila,
  alModificar: () => void,
  alternarLista: () => void,
  listaAbierta: boolean,
): HTMLElement {
  const menu = document.createElement("div");

  menu.className = "popup-perfil-acciones";

  const nombreActual = filaPerfil.accionReferencia;

  // ---------- Editar (solo si hay macro asignada) ----------
  if (nombreActual) {
    const botonEditar = crearBoton({
      texto: "Editar",
    });

    botonEditar.addEventListener("click", (eventoClick) => {
      ocultarPopup();

      abrirEditorMacro(eventoClick, contexto, filaPerfil);
    });

    menu.append(botonEditar);
  }

  // ---------- Nueva ----------
  const botonNueva = crearBoton({
    texto: "Nueva",
  });

  botonNueva.addEventListener("click", async (eventoClick) => {
    ocultarPopup();

    try {
      const resultado = await invoke<MacroArchivoResultado>("macro_nueva", {
        nombre: null,
      });

      filaPerfil.accionReferencia = resultado.nombre;

      reconstruirFila(contexto.id);

      alModificar();

      abrirEditorMacro(eventoClick, contexto, filaPerfil);
    } catch (error) {
      console.error("❌ No se pudo crear la macro:", error);
    }
  });

  menu.append(botonNueva);

  // ---------- Abrir ----------
  const botonAbrir = crearBoton({
    texto: `Abrir ${listaAbierta ? "▴" : "▾"}`,
  });

  botonAbrir.addEventListener("click", () => {
    alternarLista();
  });

  menu.append(botonAbrir);

  // ---------- Eliminar (solo si hay macro asignada) ----------
  // Doble verificación mediante el propio botón (mismo patrón que
  // "Eliminar perfil"): el primer click solo cambia el texto a
  // confirmación, el segundo ejecuta. El comportamiento de "la fila
  // que la tenía asignada queda en OFF con aviso" se mantiene tal
  // cual (ver macro_eliminar/compilador.rs), pero ya no se avisa
  // de eso en un popup de mensaje.
  if (nombreActual) {
    const botonEliminar = crearBoton({
      texto: "Eliminar",
    });

    botonEliminar.classList.add("popup-btn-peligro");

    let confirmando = false;

    botonEliminar.addEventListener("click", async () => {
      if (!confirmando) {
        confirmando = true;

        botonEliminar.textContent = "⚠️ Confirmar eliminación";

        return;
      }

      try {
        await invoke("macro_eliminar", { nombre: nombreActual });
      } catch (error) {
        console.error("❌ No se pudo eliminar la macro:", error);

        return;
      }

      filaPerfil.accionReferencia = null;

      reconstruirFila(contexto.id);

      alModificar();

      ocultarPopup();
    });

    menu.append(botonEliminar);
  }

  return menu;
}

// ======================================================
// 📋 LISTA DE MACROS (desplegada por "Abrir")
// ------------------------------------------------------
// Clic en el nombre = asigna y cierra todo el popup.
// ======================================================

function crearListaMacros(
  filaPerfil: FilaPerfil,
  contexto: ContextoFila,
  alModificar: () => void,
  alCerrarLista: () => void,
): HTMLElement {
  const lista = document.createElement("div");

  lista.className = "popup-perfil-lista";

  const cargando = document.createElement("div");

  cargando.className = "popup-perfil-nombre";

  cargando.textContent = "Cargando...";

  lista.append(cargando);

  invoke<string[]>("macro_listar")
    .then((macros) => {
      lista.innerHTML = "";

      if (macros.length === 0) {
        const vacio = document.createElement("div");

        vacio.className = "popup-perfil-nombre";

        vacio.textContent = "No hay macros guardadas todavía";

        lista.append(vacio);

        return;
      }

      macros.forEach((nombre) => {
        const botonAbrir = crearBoton({
          texto: nombre,
        });

        botonAbrir.addEventListener("click", () => {
          asignarMacro(nombre, filaPerfil, contexto, alModificar);

          ocultarPopup();
        });

        lista.append(botonAbrir);
      });
    })
    .catch((error) => {
      console.error("❌ No se pudo obtener la lista de macros:", error);

      alCerrarLista();
    });

  return lista;
}

// ======================================================
// 🔗 ASIGNAR MACRO A LA FILA
// ------------------------------------------------------
// filaPerfil.accionReferencia es el mismo campo genérico que ya lee
// compilador.rs para tipo === "macro" (ver core_perfil.ts) — no
// hace falta ningún campo nuevo.
// ======================================================

function asignarMacro(
  nombre: string,
  filaPerfil: FilaPerfil,
  contexto: ContextoFila,
  alModificar: () => void,
): void {
  filaPerfil.accionReferencia = nombre;

  reconstruirFila(contexto.id);

  alModificar();
}
