// ======================================================
// 🧩 comp_Popup_Macro_Accion
// ------------------------------------------------------
// Menú de la columna Acción del tipo "Macro" (filaPerfil.tipo ===
// "macro"), conectado desde comp_accion.ts. Desde la Etapa 8A
// concentra TODO lo que es "elegir/editar/administrar QUÉ macro" —
// antes Extra tenía "Editar" (abrir el editor); ahora vive acá,
// junto con Renombrar/Eliminar (nuevos) y Nueva/Abrir/Clonar (ya
// existían, ver Etapa 2/3). El único popup que sigue siendo aparte
// es Extra, que desde 8A pasa a ser solo el selector de
// Comportamiento (ver comp_popup_macro_extra.ts).
//
// Orden del menú (spec): Editar · Renombrar · Nueva · Abrir ·
// Clonar · Eliminar.
//
// "Abrir" despliega el listado de macros guardadas (redibuja el
// mismo popup en el lugar, mismo patrón persistente que
// comp_popup_abrir_extra.ts) — clic en un nombre asigna y cierra.
// "Clonar" clona la macro ACTUALMENTE ASIGNADA a la fila (si no hay
// ninguna, no tiene sentido — el botón queda deshabilitado).
// filaPerfil.accionReferencia sigue siendo el único dato que esta
// fila guarda sobre la macro elegida (mismo campo genérico que ya
// usa Multimedia) — nombre, no contenido.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

import { reconstruirFila } from "../ui_tabla_control";

import { confirmarPopup } from "./comp_popup_confirmar";

import { abrirEditorMacro } from "./comp_popup_macro_editor";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

// ======================================================
// 📦 RESULTADO DE macro_nueva / macro_clonar
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
      contenedor.append(crearSeparador());

      contenedor.append(
        crearListaMacros(filaPerfil, contexto, alModificar, () => {
          listaAbiertaExpandida = false;
          dibujar();
        }),
      );
    }

    mostrarPopup(contenedor, evento.clientX, evento.clientY);
  };

  dibujar();
}

// ======================================================
// 📋 MENÚ PRINCIPAL — Editar / Renombrar / Nueva / Abrir /
//     Clonar / Eliminar
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

  // ---------- Editar ----------
  const botonEditar = crearBoton({
    texto: "✏️ Editar",
    titulo: nombreActual
      ? undefined
      : "Elegí una macro primero (Abrir o Nueva)",
  });

  botonEditar.disabled = !nombreActual;

  botonEditar.addEventListener("click", (eventoClick) => {
    ocultarPopup();

    if (nombreActual) {
      abrirEditorMacro(eventoClick, contexto, filaPerfil);
    }
  });

  // ---------- Renombrar ----------
  const botonRenombrar = crearBoton({
    texto: "🏷️ Renombrar",
    titulo: nombreActual
      ? undefined
      : "Elegí una macro primero (Abrir o Nueva)",
  });

  botonRenombrar.disabled = !nombreActual;

  botonRenombrar.addEventListener("click", (eventoClick) => {
    if (!nombreActual) {
      return;
    }

    abrirFormularioNombre(
      nombreActual,
      eventoClick,
      async (nuevoNombre) => {
        const resultado = await invoke<string>("macro_renombrar", {
          nombreActual,
          nombreNuevo: nuevoNombre,
        });

        asignarMacro(resultado, filaPerfil, contexto, alModificar);
      },
      "❌ No se pudo renombrar la macro:",
    );
  });

  // ---------- Nueva ----------
  const botonNueva = crearBoton({
    texto: "🆕 Nueva",
  });

  botonNueva.addEventListener("click", (eventoClick) => {
    abrirFormularioNombre(
      "",
      eventoClick,
      async (nombre) => {
        const resultado = await invoke<MacroArchivoResultado>("macro_nueva", {
          nombre: nombre || null,
        });

        asignarMacro(resultado.nombre, filaPerfil, contexto, alModificar);
      },
      "❌ No se pudo crear la macro:",
    );
  });

  // ---------- Abrir ----------
  const botonAbrir = crearBoton({
    texto: `📂 Abrir ${listaAbierta ? "▴" : "▾"}`,
  });

  botonAbrir.addEventListener("click", () => {
    alternarLista();
  });

  // ---------- Clonar ----------
  const botonClonar = crearBoton({
    texto: "📋 Clonar",
    titulo: nombreActual ? undefined : "Elegí una macro primero para clonarla",
  });

  botonClonar.disabled = !nombreActual;

  botonClonar.addEventListener("click", (eventoClick) => {
    if (!nombreActual) {
      return;
    }

    abrirFormularioNombre(
      `${nombreActual} (copia)`,
      eventoClick,
      async (nuevoNombre) => {
        const resultado = await invoke<MacroArchivoResultado>(
          "macro_clonar",
          {
            nombreOrigen: nombreActual,

            nombreNuevo: nuevoNombre,
          },
        );

        asignarMacro(resultado.nombre, filaPerfil, contexto, alModificar);
      },
      "❌ No se pudo clonar la macro:",
    );
  });

  // ---------- Eliminar ----------
  const botonEliminar = crearBoton({
    texto: "🗑️ Eliminar",
    titulo: nombreActual ? undefined : "Elegí una macro primero",
  });

  botonEliminar.disabled = !nombreActual;

  botonEliminar.addEventListener("click", async (eventoClick) => {
    if (!nombreActual) {
      return;
    }

    const confirmado = await confirmarPopup(
      `¿Eliminar la macro "${nombreActual}"? Si alguna fila la tiene asignada, quedará en OFF con aviso hasta que se le asigne otra.`,
      eventoClick,
    );

    if (!confirmado) {
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
  });

  menu.append(
    botonEditar,
    botonRenombrar,
    botonNueva,
    botonAbrir,
    botonClonar,
    botonEliminar,
  );

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
// ✏️ FORMULARIO DE NOMBRE
// ------------------------------------------------------
// Compartido entre Renombrar / Nueva / Clonar — mismo patrón que
// abrirFormularioRenombrar() en comp_popup_perfil.ts.
// ======================================================

function abrirFormularioNombre(
  valorInicial: string,
  evento: MouseEvent,
  confirmar: (nombre: string) => Promise<void>,
  mensajeError: string,
): void {
  const contenedor = document.createElement("div");

  contenedor.className = "popup-perfil-renombrar";

  const input = document.createElement("input");

  input.className = "popup-input";

  input.type = "text";

  input.value = valorInicial;

  input.placeholder = "macro_001";

  const botones = document.createElement("div");

  botones.className = "popup-confirmar-botones";

  const botonCancelar = crearBoton({
    texto: "Cancelar",
  });

  const botonGuardar = crearBoton({
    texto: "Guardar",
  });

  const aceptar = async (): Promise<void> => {
    try {
      await confirmar(input.value.trim());
    } catch (error) {
      console.error(mensajeError, error);

      return;
    } finally {
      ocultarPopup();
    }
  };

  botonGuardar.addEventListener("click", aceptar);

  botonCancelar.addEventListener("click", () => {
    ocultarPopup();
  });

  input.addEventListener("keydown", (evento) => {
    if (evento.key === "Enter") {
      aceptar();
    }

    if (evento.key === "Escape") {
      ocultarPopup();
    }
  });

  botones.append(botonCancelar, botonGuardar);

  contenedor.append(input, botones);

  mostrarPopup(contenedor, evento.clientX, evento.clientY);

  input.focus();

  input.select();
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

// ======================================================
// SEPARADOR
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "popup-perfil-separador";

  return separador;
}
