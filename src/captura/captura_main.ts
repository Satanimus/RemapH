// ======================================================
// 📌 captura_Main
// ------------------------------------------------------
// Punto de entrada de la ventana overlay de "Click en
// coordenada" (captura.html — página independiente, ver
// vite.config.ts). Todo el cálculo de %H/%V y offsets vive
// ACÁ, en TS — Rust solo entrega datos crudos (cursor,
// rect+título de ventana activa) vía back_coordenada.rs.
//
// Config activa (ubicación/modo/punto de referencia) se lee
// UNA sola vez al cargar desde obtener_config_captura_activa
// (fijada por comandos.rs justo antes de crear esta ventana
// — sin carrera posible). Si el usuario cambia esa config en
// el popup principal mientras esta ventana sigue abierta, el
// popup la cierra automáticamente (ver comp_popup_coordenada.ts)
// — acá nunca hace falta releer la config a mitad de camino.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import "../styles/styl_variables.css";
import "./captura.css";

// ======================================================
// 🧭 TIPOS
// ======================================================

interface ConfigCaptura {
  ubicacion: "absoluta" | "relativa_cursor" | "relativa_ventana";

  modoVentana: "porcentaje" | "pixeles";

  puntoReferencia: "sup_izq" | "sup_der" | "centro" | "inf_izq" | "inf_der";
}

interface VentanaActiva {
  titulo: string;

  x: number;

  y: number;

  ancho: number;

  alto: number;
}

// ======================================================
// 🏗️ ARMAR DOM
// ======================================================

const raiz = document.getElementById("captura")!;

const card = document.createElement("div");
card.className = "captura-card";

const header = document.createElement("div");
header.className = "captura-header";
header.setAttribute("data-tauri-drag-region", "");

const icono = document.createElement("span");
icono.className = "captura-header-icono";
icono.textContent = "⠿";

const titulo = document.createElement("span");
titulo.className = "captura-header-titulo";
titulo.textContent = "MODO CAPTURA";

const botonCancelar = document.createElement("button");
botonCancelar.className = "captura-cancelar";
botonCancelar.textContent = "Cancelar";

header.append(icono, titulo, botonCancelar);

const cuerpo = document.createElement("div");
cuerpo.className = "captura-cuerpo";

card.append(header, cuerpo);
raiz.append(card);

// ======================================================
// 🩺 DIAGNÓSTICO EN PANTALLA
// ------------------------------------------------------
// F12/devtools no siempre engancha en una ventana sin
// decoraciones — en vez de depender de la consola, cualquier
// error o timeout se escribe directo en el cuerpo de la
// ventana para poder verlo sin herramientas externas.
// TODO: sacar este bloque una vez confirmado que la ventana
// funciona de punta a punta.
// ======================================================

function mostrarDiagnostico(texto: string): void {
  const linea = document.createElement("div");
  linea.className = "captura-linea";
  linea.style.color = "#FF6B6B";
  linea.style.wordBreak = "break-word";
  linea.textContent = texto;
  cuerpo.append(linea);
}

function conTimeout<T>(
  promesa: Promise<T>,
  ms: number,
  etiqueta: string,
): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(
        new Error(
          `Timeout (${ms}ms) esperando "${etiqueta}" — el invoke nunca resolvió ni rechazó.`,
        ),
      );
    }, ms);

    promesa.then(
      (valor) => {
        clearTimeout(timer);
        resolve(valor);
      },
      (error) => {
        clearTimeout(timer);
        reject(error);
      },
    );
  });
}

// ======================================================
// 🚪 CERRAR (sin guardar)
// ======================================================

botonCancelar.addEventListener("click", () => {
  mostrarDiagnostico("Cancelar clickeado, invocando cierre...");
  cerrar();
});

function cerrar(): void {
  detenerPolling();

  invoke("cerrar_ventana_captura_coordenada")
    .then(() =>
      mostrarDiagnostico("cerrar_ventana_captura_coordenada resolvió OK."),
    )
    .catch((error) => mostrarDiagnostico(`Error al cerrar: ${String(error)}`));
}

// ======================================================
// 📐 PUNTO DE REFERENCIA → COORDENADA ABSOLUTA
// ------------------------------------------------------
// Espejo de punto_referencia_absoluto() en back_coordenada.rs.
// ======================================================

function puntoReferenciaAbsoluto(
  referencia: ConfigCaptura["puntoReferencia"],
  ventana: VentanaActiva,
): { x: number; y: number } {
  switch (referencia) {
    case "sup_izq":
      return { x: ventana.x, y: ventana.y };

    case "sup_der":
      return { x: ventana.x + ventana.ancho, y: ventana.y };

    case "centro":
      return {
        x: ventana.x + ventana.ancho / 2,
        y: ventana.y + ventana.alto / 2,
      };

    case "inf_izq":
      return { x: ventana.x, y: ventana.y + ventana.alto };

    case "inf_der":
      return { x: ventana.x + ventana.ancho, y: ventana.y + ventana.alto };
  }
}

// ======================================================
// 🏷️ TEXTO PUNTO DE REFERENCIA
// ======================================================

function textoPuntoReferencia(
  referencia: ConfigCaptura["puntoReferencia"],
): string {
  switch (referencia) {
    case "sup_izq":
      return "Sup-Izq";
    case "sup_der":
      return "Sup-Der";
    case "centro":
      return "Centro";
    case "inf_izq":
      return "Inf-Izq";
    case "inf_der":
      return "Inf-Der";
  }
}

// ======================================================
// ⏱️ ESTADO DE POLLING
// ======================================================

let intervaloId: ReturnType<typeof setInterval> | null = null;

function detenerPolling(): void {
  if (intervaloId !== null) {
    clearInterval(intervaloId);
    intervaloId = null;
  }
}

// Estado del paso 1/2 exclusivo de "Relativa a cursor".
let pasoRelativaCursor: 1 | 2 = 1;
let origenRelativaCursor: { x: number; y: number } | null = null;

// ======================================================
// 💾 GUARDAR RESULTADO Y CERRAR
// ======================================================

function guardarYcerrar(x: number, y: number): void {
  detenerPolling();

  invoke("guardar_resultado_coordenada", { x, y }).catch(() => {});

  cuerpo.innerHTML = "";
  const feedback = document.createElement("div");
  feedback.className = "captura-guardado";
  feedback.textContent = "✅ Guardado";
  cuerpo.append(feedback);

  setTimeout(() => {
    invoke("cerrar_ventana_captura_coordenada").catch(() => {});
  }, 500);
}

// ======================================================
// 🏁 INICIAR
// ======================================================

async function iniciar(): Promise<void> {
  mostrarDiagnostico("Iniciando — consultando config activa...");

  let configCruda: {
    ubicacion: string;
    modo_ventana: string;
    punto_referencia: string;
  } | null;

  try {
    configCruda = await conTimeout(
      invoke("obtener_config_captura_activa"),
      3000,
      "obtener_config_captura_activa",
    );
  } catch (error) {
    mostrarDiagnostico(`FALLÓ obtener_config_captura_activa: ${String(error)}`);
    return;
  }

  mostrarDiagnostico(`Config recibida: ${JSON.stringify(configCruda)}`);

  // No debería pasar (comandos.rs fija la config antes de crear esta
  // ventana) — pero si pasa, no hay nada coherente que mostrar.
  if (!configCruda) {
    cerrar();
    return;
  }

  const config: ConfigCaptura = {
    ubicacion: configCruda.ubicacion as ConfigCaptura["ubicacion"],
    modoVentana: configCruda.modo_ventana as ConfigCaptura["modoVentana"],
    puntoReferencia:
      configCruda.punto_referencia as ConfigCaptura["puntoReferencia"],
  };

  let intervaloMs = 100;

  try {
    intervaloMs = await conTimeout(
      invoke<number>("obtener_intervalo_captura_coordenada"),
      3000,
      "obtener_intervalo_captura_coordenada",
    );
  } catch (error) {
    mostrarDiagnostico(
      `FALLÓ obtener_intervalo_captura_coordenada (uso 100ms): ${String(error)}`,
    );
  }

  intervaloId = setInterval(() => actualizar(config), intervaloMs);
  actualizar(config);
}

// ======================================================
// 🔄 ACTUALIZAR (un tick de polling)
// ======================================================

async function actualizar(config: ConfigCaptura): Promise<void> {
  let cursor: [number, number] | null;
  let ventana: VentanaActiva | null;
  let guardar: boolean;

  try {
    [cursor, ventana, guardar] = await conTimeout(
      Promise.all([
        invoke<[number, number]>("obtener_cursor_captura"),
        invoke<VentanaActiva | null>("obtener_ventana_activa_captura"),
        invoke<boolean>("consultar_guardado_coordenada"),
      ]),
      3000,
      "polling (cursor/ventana/guardado)",
    );
  } catch (error) {
    detenerPolling();
    mostrarDiagnostico(`FALLÓ el polling: ${String(error)}`);
    return;
  }

  if (!cursor) {
    return;
  }

  const [cursorX, cursorY] = cursor;

  cuerpo.innerHTML = "";

  switch (config.ubicacion) {
    case "absoluta": {
      dibujarLinea("Presione la tecla configurada para guardar posición");
      dibujarLinea(`X: ${cursorX}  Y: ${cursorY}`, true);

      if (guardar) {
        guardarYcerrar(cursorX, cursorY);
      }

      break;
    }

    case "relativa_cursor": {
      if (pasoRelativaCursor === 1) {
        dibujarLinea("Origen: presione la tecla para marcar origen");
        dibujarLinea(`X: ${cursorX}  Y: ${cursorY}`, true);

        if (guardar) {
          origenRelativaCursor = { x: cursorX, y: cursorY };
          pasoRelativaCursor = 2;
        }
      } else {
        const origen = origenRelativaCursor!;
        const offsetX = cursorX - origen.x;
        const offsetY = cursorY - origen.y;

        dibujarLinea("Destino: presione la tecla para marcar destino");
        dibujarLinea(
          `X: ${offsetX >= 0 ? "+" : ""}${offsetX}  Y: ${offsetY >= 0 ? "+" : ""}${offsetY}`,
          true,
        );

        if (guardar) {
          guardarYcerrar(offsetX, offsetY);
        }
      }

      break;
    }

    case "relativa_ventana": {
      if (!ventana) {
        dibujarLinea("Ventana activa: [fuera de ventana]");
        dibujarLinea("Presione la tecla configurada para guardar posición");

        break;
      }

      dibujarLinea(`Ventana activa: ${ventana.titulo || "(sin título)"}`);

      if (config.modoVentana === "porcentaje") {
        const h = ((cursorX - ventana.x) / ventana.ancho) * 100;
        const v = ((cursorY - ventana.y) / ventana.alto) * 100;

        dibujarLinea(`H: ${h.toFixed(1)}%  V: ${v.toFixed(1)}%`, true);

        if (guardar) {
          guardarYcerrar(h, v);
        }
      } else {
        dibujarLinea(
          `Referencia: ${textoPuntoReferencia(config.puntoReferencia)}`,
        );

        const base = puntoReferenciaAbsoluto(config.puntoReferencia, ventana);
        const offsetX = cursorX - base.x;
        const offsetY = cursorY - base.y;

        dibujarLinea(`X: ${offsetX}  Y: ${offsetY}`, true);

        if (guardar) {
          guardarYcerrar(offsetX, offsetY);
        }
      }

      break;
    }
  }
}

// ======================================================
// 🖊️ DIBUJAR LÍNEA DE TEXTO EN EL CUERPO
// ======================================================

function dibujarLinea(texto: string, destacada = false): void {
  const linea = document.createElement("div");
  linea.className = destacada ? "captura-linea captura-valor" : "captura-linea";
  linea.textContent = texto;
  cuerpo.append(linea);
}

iniciar();
