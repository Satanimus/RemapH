// ======================================================
// 📂🗂️ comp_Popup_Abrir_Con
// ------------------------------------------------------
// Popup de selección de "Abrir con..." del tipo "Abrir Archivo/App"
// (filaPerfil.tipo === "abrir"), abierto desde el botón "Abrir con"
// dentro del popup Extra (ver comp_popup_abrir_extra.ts) cuando
// abrirAccion.ruta NO es un .exe.
//
// Mismo estilo que comp_popup_app.ts: caja oscura con un listado de
// botones (ícono + nombre), pero acá la fuente es el registro de
// Windows en vez de los procesos corriendo — obtener_programas_abrir_con()
// (recientes de esa extensión primero, luego instalados, ver
// back_registro.rs) — más una opción fija al final, "Examinar...",
// que cae al selector manual ya existente (seleccionar_archivo
// filtrado a .exe) para cuando el programa deseado no aparece
// listado.
//
// El ícono de cada ítem NO viaja en la respuesta del listado — se
// pide aparte por ítem con obtener_icono_ruta() (mismo patrón que ya
// usa la columna App), para no bloquear el popup completo esperando
// todos los íconos antes de mostrar nada.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui_tabla_control";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import { extensionDeRuta } from "../../core/core_abrir";

// ======================================================
// 📦 MODELOS BACKEND
// ======================================================

interface IconoJson {
  ancho: number;

  alto: number;

  pixeles: string;
}

interface ProgramaJson {
  nombre: string;

  ruta: string;
}

// ======================================================
// 🎨 ÍCONO — FALLBACK Y REAL
// ------------------------------------------------------
// Mismo patrón de conversión base64 → canvas RGBA que ya usan
// comp_popup_app.ts / comp_popup_abrir_accion.ts.
// ======================================================

function crearIconoFallback(): HTMLElement {
  const icono = document.createElement("span");

  icono.className = "app-icono-fallback";

  icono.textContent = "▣";

  return icono;
}

function crearIcono(datos: IconoJson): HTMLElement {
  const canvas = document.createElement("canvas");

  canvas.width = datos.ancho;

  canvas.height = datos.alto;

  const contexto = canvas.getContext("2d");

  if (!contexto) {
    return crearIconoFallback();
  }

  const pixeles = Uint8ClampedArray.from(atob(datos.pixeles), (caracter) =>
    caracter.charCodeAt(0),
  );

  contexto.putImageData(new ImageData(pixeles, datos.ancho, datos.alto), 0, 0);

  canvas.className = "app-icono";

  return canvas;
}

// ======================================================
// 🔘 BOTÓN DE PROGRAMA
// ------------------------------------------------------
// Ícono en fallback hasta que resuelve obtener_icono_ruta() (mismo
// patrón asíncrono que crearAccionAbrir() en
// comp_popup_abrir_accion.ts) — no bloquea el listado esperando el
// ícono de cada ítem antes de mostrarlo.
// ======================================================

function crearBotonPrograma(
  programa: ProgramaJson,
  seleccionar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn app-popup-programa";

  const icono = crearIconoFallback();

  boton.append(icono);

  const nombre = document.createElement("span");

  nombre.className = "app-popup-nombre";

  nombre.textContent = programa.nombre;

  boton.append(nombre);

  boton.title = programa.ruta;

  invoke<IconoJson | null>("obtener_icono_ruta", { ruta: programa.ruta })
    .then((iconoJson) => {
      if (!iconoJson) {
        return;
      }

      boton.replaceChild(crearIcono(iconoJson), icono);
    })
    .catch(() => {});

  boton.addEventListener("click", () => {
    seleccionar();

    ocultarPopup();
  });

  return boton;
}

// ======================================================
// 🔍 EXAMINAR... (selector manual, fija al final del listado)
// ------------------------------------------------------
// Mismo comando que ya usaba el botón "Abrir con" antes de esta
// etapa (seleccionar_archivo filtrado a .exe) — queda como vía de
// escape para cuando el programa deseado no aparece en el listado
// del registro.
// ======================================================

function crearBotonExaminar(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alSeleccionar?: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn app-popup-programa";

  const icono = document.createElement("span");

  icono.className = "app-popup-global-icono";

  icono.textContent = "📂";

  boton.append(icono);

  const nombre = document.createElement("span");

  nombre.className = "app-popup-nombre";

  nombre.textContent = "Examinar...";

  boton.append(nombre);

  boton.addEventListener("click", async () => {
    ocultarPopup();

    const ruta = await invoke<string | null>("seleccionar_archivo", {
      extensiones: ["exe"],
    });

    if (!ruta) {
      return;
    }

    filaPerfil.abrirExtra.abrirCon = ruta;

    reconstruirFila(contexto.id);

    alSeleccionar?.();
  });

  return boton;
}

// ======================================================
// ➖ SEPARADOR (mismo estilo que el resto de los popups)
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

// ======================================================
// 📂🗂️ ABRIR POPUP "ABRIR CON..."
// ------------------------------------------------------
// Sin filtro Principales/Otros (a diferencia de comp_popup_app.ts):
// acá la lista ya viene ordenada por relevancia desde el backend
// (recientes de esa extensión primero, luego instalados) y suele ser
// mucho más corta que "todos los procesos corriendo" — no amerita
// dividirla en dos pestañas.
// ======================================================

export async function abrirPopupAbrirCon(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alSeleccionar?: () => void,
): Promise<void> {
  const extension = extensionDeRuta(filaPerfil.abrirAccion.ruta);

  const programas = await invoke<ProgramaJson[]>(
    "obtener_programas_abrir_con",
    { extension },
  );

  const popup = document.createElement("div");

  popup.className = "app-popup";

  const titulo = document.createElement("span");

  titulo.className = "app-popup-lista-titulo";

  titulo.textContent = "Abrir con:";

  popup.append(titulo);

  const caja = document.createElement("div");

  caja.className = "popup-caja-interna app-popup-lista-caja";

  const lista = document.createElement("div");

  lista.className = "app-popup-lista";

  programas.forEach((programa) => {
    lista.append(
      crearBotonPrograma(programa, () => {
        filaPerfil.abrirExtra.abrirCon = programa.ruta;

        reconstruirFila(contexto.id);

        alSeleccionar?.();
      }),
    );
  });

  caja.append(lista);

  popup.append(caja);

  popup.append(crearSeparador());

  popup.append(crearBotonExaminar(contexto, filaPerfil, alSeleccionar));

  mostrarPopup(popup, evento.clientX, evento.clientY);
}
