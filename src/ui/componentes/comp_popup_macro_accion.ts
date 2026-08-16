// ======================================================
// 🧩 comp_Popup_Macro_Accion
// ------------------------------------------------------
// Popup de la columna Acción del tipo "Macro" (filaPerfil.tipo
// === "macro"), conectado desde comp_accion.ts en la Etapa 3.
// Mismo espíritu que comp_popup_perfil.ts (lista + Nueva/
// Clonar) pero acotado a una sola fila en vez de a todo el
// perfil — acá no hay "macro actual" ni recompilación, elegir
// una macro es solo guardar su nombre en
// filaPerfil.accionReferencia (mismo campo genérico que ya usa
// compilador.rs para tipo === "macro", ver core_perfil.ts).
//
// El popup editor completo (Tipo/Acción/Extra, Etapa 5) es
// aparte — acá solo se decide A CUÁL macro apunta la fila.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

import { reconstruirFila } from "../ui_tabla_control";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

// ======================================================
// 📦 RESULTADO DE macro_nueva / macro_clonar
// ------------------------------------------------------
// Espejo de MacroArchivoJson (macro_json.rs) — acá solo
// importa `nombre`, el resto (pasos) lo consume el editor de
// la Etapa 5.
// ======================================================

interface MacroArchivoResultado {
  nombre: string;
}

// ======================================================
// 🚪 ABRIR POPUP
// ======================================================

export async function abrirPopupMacroAccion(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): Promise<void> {
  let macros: string[];

  try {
    macros = await invoke<string[]>("macro_listar");
  } catch (error) {
    console.error("❌ No se pudo obtener la lista de macros:", error);

    return;
  }

  const contenedor = document.createElement("div");

  contenedor.className = "popup-perfil";

  contenedor.append(
    crearListaMacros(macros, filaPerfil, contexto, alModificar),

    crearSeparador(),

    crearAcciones(filaPerfil, contexto, alModificar),
  );

  mostrarPopup(contenedor, evento.clientX, evento.clientY);
}

// ======================================================
// 📋 LISTA DE MACROS
// ------------------------------------------------------
// Clic en el nombre = Abrir Macro (asigna y cierra). El ícono
// 📋 aparte = Clonar Macro (pide nombre nuevo, sin tocar la
// selección de la fila hasta confirmar).
// ======================================================

function crearListaMacros(
  macros: string[],
  filaPerfil: FilaPerfil,
  contexto: ContextoFila,
  alModificar: () => void,
): HTMLElement {
  const lista = document.createElement("div");

  lista.className = "popup-perfil-lista";

  if (macros.length === 0) {
    const vacio = document.createElement("div");

    vacio.className = "popup-perfil-nombre";

    vacio.textContent = "No hay macros guardadas todavía";

    lista.append(vacio);

    return lista;
  }

  macros.forEach((nombre) => {
    const fila = document.createElement("div");

    fila.className = "popup-perfil-item";

    const botonAbrir = crearBoton({
      texto: nombre,
    });

    botonAbrir.addEventListener("click", () => {
      asignarMacro(nombre, filaPerfil, contexto, alModificar);

      ocultarPopup();
    });

    const botonClonar = crearBoton({
      texto: "📋",

      titulo: "Clonar esta macro",
    });

    botonClonar.addEventListener("click", (evento) => {
      abrirFormularioNombre(
        `${nombre} (copia)`,
        evento,
        async (nuevoNombre) => {
          const resultado = await invoke<MacroArchivoResultado>(
            "macro_clonar",
            {
              nombreOrigen: nombre,

              nombreNuevo: nuevoNombre,
            },
          );

          asignarMacro(resultado.nombre, filaPerfil, contexto, alModificar);
        },
        "❌ No se pudo clonar la macro:",
      );
    });

    fila.append(botonAbrir, botonClonar);

    lista.append(fila);
  });

  return lista;
}

// ======================================================
// ➕ ACCIONES (Nueva Macro)
// ======================================================

function crearAcciones(
  filaPerfil: FilaPerfil,
  contexto: ContextoFila,
  alModificar: () => void,
): HTMLElement {
  const acciones = document.createElement("div");

  acciones.className = "popup-perfil-acciones";

  const botonNueva = crearBoton({
    texto: "Nueva Macro",
  });

  botonNueva.addEventListener("click", (evento) => {
    abrirFormularioNombre(
      "",
      evento,
      async (nombre) => {
        const resultado = await invoke<MacroArchivoResultado>("macro_nueva", {
          nombre: nombre || null,
        });

        asignarMacro(resultado.nombre, filaPerfil, contexto, alModificar);
      },
      "❌ No se pudo crear la macro:",
    );
  });

  acciones.append(botonNueva);

  return acciones;
}

// ======================================================
// ✏️ FORMULARIO DE NOMBRE
// ------------------------------------------------------
// Compartido entre "Nueva Macro" y "Clonar" (ícono 📋) — mismo
// patrón que abrirFormularioRenombrar() en comp_popup_perfil.ts.
// valorInicial vacío para Nueva Macro (placeholder sugiere el
// nombre automático, el backend lo resuelve si se deja vacío).
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
// filaPerfil.accionReferencia es el mismo campo genérico que
// ya lee compilador.rs para tipo === "macro" (ver
// core_perfil.ts) — no hace falta ningún campo nuevo.
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
