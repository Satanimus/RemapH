// ======================================================
// 📂 comp_Popup_Abrir_Accion
// ------------------------------------------------------
// Botón + popup de la columna Acción del tipo "Abrir Archivo/App"
// (filaPerfil.tipo === "abrir"), conectado desde comp_accion.ts.
//
// El botón muestra el ícono real de la ruta elegida (vía
// obtener_icono_ruta, mismo comando que usa comandos.rs para
// carpeta/documento/.lnk/.exe) + el nombre calculado
// (textoAbrirAccion(), ver core_abrir.ts) + tooltip con la ruta
// completa. "Seleccionar..." con ícono de carpeta genérico hasta que
// se elige algo.
//
// Al click, un pequeño popup ofrece Archivo/Carpeta antes de invocar
// el selector nativo correspondiente — rfd no tiene un diálogo que
// combine ambos en una sola ventana (ver comandos.rs::
// seleccionar_archivo() / seleccionar_carpeta()).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import { textoAbrirAccion } from "../core/core_abrir";

// ======================================================
// 📦 MODELO ÍCONO (mismo shape que comandos.rs::IconoJson)
// ======================================================

interface IconoJson {
  ancho: number;

  alto: number;

  pixeles: string;
}

// ======================================================
// 🎨 ÍCONO — FALLBACK Y REAL
// ------------------------------------------------------
// Mismo patrón de conversión base64 → canvas RGBA que ya usan
// comp_popup_app.ts / comp_controles.ts::crearApp() — acá se repite
// en vez de compartirse, siguiendo el mismo criterio que esos dos
// (cada punto de uso es dueño de su propio render de ícono).
// ======================================================

function crearIconoFallback(): HTMLElement {
  const icono = document.createElement("span");

  icono.className = "app-icono-fallback";

  icono.textContent = "📂";

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
// 🔘 BOTÓN DE ACCIÓN
// ======================================================

export function crearAccionAbrir(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn capturador";

  const icono = crearIconoFallback();

  boton.append(icono);

  const nombre = document.createElement("span");

  nombre.textContent = textoAbrirAccion(
    filaPerfil.abrirAccion,
    filaPerfil.abrirExtra,
  );

  boton.append(nombre);

  boton.title = filaPerfil.abrirAccion.ruta ?? "Seleccionar...";

  if (filaPerfil.abrirAccion.ruta) {
    invoke<IconoJson | null>("obtener_icono_ruta", {
      ruta: filaPerfil.abrirAccion.ruta,
    })
      .then((iconoJson) => {
        if (!iconoJson) {
          return;
        }

        boton.replaceChild(crearIcono(iconoJson), icono);
      })
      .catch(() => {});
  }

  boton.addEventListener("click", (evento) => {
    abrirPopupSeleccionRuta(evento, contexto, filaPerfil);

    alModificar();
  });

  return boton;
}

// ======================================================
// 📄📁 POPUP DE SELECCIÓN — ARCHIVO / CARPETA
// ------------------------------------------------------
// rfd no ofrece un único diálogo nativo que combine archivo+carpeta
// (ver comandos.rs) — se ofrece la elección acá antes de invocar el
// selector correspondiente. Sin filtro de extensión: cualquier
// archivo es válido (documento, .exe, .lnk), la distinción la hace
// runtime.rs al ejecutar según su propia extensión.
// ======================================================

function abrirPopupSeleccionRuta(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const lista = document.createElement("div");

  lista.className = "popup-lista";

  const botonArchivo = document.createElement("button");

  botonArchivo.className = "ui-btn";
  botonArchivo.textContent = "📄 Archivo...";

  botonArchivo.addEventListener("click", async () => {
    ocultarPopup();

    const ruta = await invoke<string | null>("seleccionar_archivo", {
      extensiones: null,
    });

    aplicarRuta(ruta, contexto, filaPerfil);
  });

  const botonCarpeta = document.createElement("button");

  botonCarpeta.className = "ui-btn";
  botonCarpeta.textContent = "📁 Carpeta...";

  botonCarpeta.addEventListener("click", async () => {
    ocultarPopup();

    const ruta = await invoke<string | null>("seleccionar_carpeta");

    aplicarRuta(ruta, contexto, filaPerfil);
  });

  lista.append(botonArchivo, botonCarpeta);

  mostrarPopup(lista, evento.clientX, evento.clientY);
}

function aplicarRuta(
  ruta: string | null,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  if (!ruta) {
    return;
  }

  filaPerfil.abrirAccion.ruta = ruta;

  // abrirCon/argumento fueron elegidos para el archivo ANTERIOR —
  // abrirCon puede no ser un programa válido para la nueva extensión
  // (ej. se eligió Notepad++ para un .txt y ahora la Acción pasa a
  // ser un .jpg) y argumento es específico del .exe anterior. Se
  // limpian los dos al cambiar la ruta, igual que crearTipo() en
  // comp_controles.ts resetea abrirAccion/abrirExtra completos al
  // cambiar de Tipo — acá el criterio es más fino (solo lo que deja
  // de tener sentido), porque cambiar la Acción no reinicia el resto
  // de la fila (Iniciar/Instancias siguen siendo válidos).
  filaPerfil.abrirExtra.abrirCon = null;

  filaPerfil.abrirExtra.argumento = "";

  reconstruirFila(contexto.id);
}
