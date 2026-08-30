// ======================================================
// 📌 vent_captura_Main
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
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";

import type { Entrada } from "../core/core_entrada";
import { aplicarOverridesApariencia } from "../core/core_apariencia";

import "../styles/styl_variables.css";
import "../styles/styl_captura.css";

void aplicarOverridesApariencia();

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
icono.setAttribute("data-tauri-drag-region", "");

const titulo = document.createElement("span");
titulo.className = "captura-header-titulo";
titulo.textContent = "MODO CAPTURA";
titulo.setAttribute("data-tauri-drag-region", "");

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

// Regla 7: Esc cancela esta ventana (equivalente a Cancelar).
document.addEventListener("keydown", (evento) => {
  if (evento.key === "Escape") cerrar();
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

// Tecla de guardado configurada (config.rs — "F1" por defecto). Se
// consulta una sola vez al cargar, igual que el resto de la config
// activa — se usa en los textos de instrucción en vez del genérico
// "la tecla configurada".
//
// obtener_tecla_guardar_coordenada devuelve AtajoCapturaUI (objeto
// {modificadores, gatillo}, no un string plano — de ahí salía
// "[object Object]" cuando se interpolaba el objeto directo en el
// texto). Acá se lo formatea a texto tipo "Ctrl+F1" antes de
// guardarlo en teclaGuardar.
interface AtajoCapturaUI {
  modificadores: Entrada[];
  gatillo: Entrada;
}

function atajoCapturaATexto(atajo: AtajoCapturaUI): string {
  return [
    ...atajo.modificadores.map((entrada) => entrada.nombre),
    atajo.gatillo.nombre,
  ].join("+");
}

let teclaGuardar = "F1";

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
// 👁️ MODO PREVISUALIZACIÓN (Etapa F)
// ------------------------------------------------------
// Marcador "⊙" en vivo, sin header/Cancelar/texto de
// diagnóstico. El destino ya viene calculado desde Rust
// (obtener_destino_preview_coordenada, por id) — acá solo se
// reposiciona la ventana para centrar el marcador sobre el
// punto. Mitades fijas: la ventana overlay siempre se crea
// con inner_size(320, 120) (ver abrir_ventana_preview_coordenada
// en comandos.rs), no es resizable.
//
// x/y llegan en coordenadas FÍSICAS (mismo sistema que
// GetCursorPos/GetWindowRect en back_coordenada.rs — ver el
// comentario largo sobre esto en back_menu_express.rs::
// monitor_para_punto). Mezclar eso con LogicalPosition rompe el
// posicionamiento en cualquier monitor con escalado != 100%: por
// eso acá se posiciona con PhysicalPosition, convirtiendo la
// mitad de ventana (lógica, 160x60) a físico vía scaleFactor().
//
// El marcador es arrastrable (Regla 17) — ver
// activarArrastreMarcador() más abajo.
// ======================================================

const MITAD_ANCHO_PREVIEW_LOGICO = 160;
const MITAD_ALTO_PREVIEW_LOGICO = 60;

async function iniciarPreview(id: string, numero: number): Promise<void> {
  raiz.innerHTML = "";

  const marcador = document.createElement("div");
  marcador.className = "captura-marcador-preview";
  // Círculo con punto central (Regla 5) en vez de la cruz "X": un
  // SVG simétrico centrado en (12,12) cae exacto en el centro de la
  // ventana, que es donde se posiciona el destino calculado.
  marcador.innerHTML =
    '<svg viewBox="0 0 24 24" width="48" height="48"><circle cx="12" cy="12" r="9" /><circle class="captura-marcador-punto" cx="12" cy="12" r="1.6" /></svg>';
  raiz.append(marcador);

  // Bug 2: el número de la fila que creó el marcador se muestra
  // siempre, apegado abajo dentro del círculo. Se crea ACÁ, antes de
  // cualquier "await" — si alguno de los invoke() de abajo (config
  // de intervalo, scaleFactor) tarda o falla, no debe arrastrar
  // consigo la aparición del número. Va como hermano de "marcador"
  // (no adentro): .captura-marcador-preview tiene opacity:0.4 para
  // que el círculo quede discreto, y ese opacity diluye TODO su
  // subárbol por igual — metido adentro, el número quedaba
  // prácticamente invisible. Como hermano, con position:absolute y
  // sin ancestro posicionado de por medio, cae en el mismo centro
  // (ver .captura-marcador-numero en styl_captura.css) pero a
  // opacidad plena.
  const numeroSpan = document.createElement("span");
  numeroSpan.className = "captura-marcador-numero";
  numeroSpan.textContent = String(numero);
  raiz.append(numeroSpan);

  let intervaloMs = 100;

  try {
    intervaloMs = await conTimeout(
      invoke<number>("obtener_intervalo_captura_coordenada"),
      3000,
      "obtener_intervalo_captura_coordenada",
    );
  } catch {
    // Sin diagnóstico visible en este modo — se sigue con 100ms.
  }

  const ventana = getCurrentWindow();
  const escala = await ventana.scaleFactor();

  // Bug 2: zona de arrastre chica (30px de diámetro, ~15px de radio
  // desde el centro) en vez de todo el marcador (que ocupa la ventana
  // entera, 320x120) — antes la mano "grab" aparecía en cualquier
  // punto de la ventana, muy lejos del círculo dibujado.
  const zonaArrastre = document.createElement("div");
  zonaArrastre.className = "captura-marcador-zona-arrastre";
  marcador.append(zonaArrastre);

  activarArrastreMarcador(zonaArrastre, id, ventana, escala, intervaloMs);

  iniciarPollingPreview(ventana, escala, id, intervaloMs);
}

function iniciarPollingPreview(
  ventana: ReturnType<typeof getCurrentWindow>,
  escala: number,
  id: string,
  intervaloMs: number,
): void {
  intervaloId = setInterval(
    () => void actualizarPreview(ventana, escala, id),
    intervaloMs,
  );
  void actualizarPreview(ventana, escala, id);
}

async function actualizarPreview(
  ventana: ReturnType<typeof getCurrentWindow>,
  escala: number,
  id: string,
): Promise<void> {
  let destino: [number, number] | null;

  try {
    destino = await conTimeout(
      invoke<[number, number] | null>("obtener_destino_preview_coordenada", {
        id,
      }),
      3000,
      "obtener_destino_preview_coordenada",
    );
  } catch {
    detenerPolling();
    return;
  }

  if (!destino) {
    return;
  }

  const [x, y] = destino;

  await ventana.setPosition(
    new PhysicalPosition(
      x - MITAD_ANCHO_PREVIEW_LOGICO * escala,
      y - MITAD_ALTO_PREVIEW_LOGICO * escala,
    ),
  );
}

// ======================================================
// 🖱️ ARRASTRAR EL MARCADOR (Regla 17)
// ------------------------------------------------------
// El elemento recibido es la zona de arrastre chica (ver
// iniciarPreview), no el marcador completo — así el punto de
// mousedown ya cae siempre muy cerca del centro real, y alcanza
// con startDragging() directo (deja que el SO mueva la ventana
// seguiendo el mouse). Antes se encadenaba un setPosition() +
// .then(startDragging()) para "saltar" el punto clickeado al
// centro — esa segunda vuelta async (después de esperar la
// respuesta de setPosition) llegaba tarde: el SO ya no tomaba el
// mousedown como válido para iniciar el arrastre nativo, y la
// ventana no se movía. Mientras se arrastra se detiene el
// polling (si no, cada tick de actualizarPreview movería la
// ventana de vuelta al punto guardado, que todavía no cambió,
// peleando con el arrastre); se retoma al soltar, ya con el
// valor nuevo persistido.
// ======================================================

function activarArrastreMarcador(
  zonaArrastre: HTMLElement,
  id: string,
  ventana: ReturnType<typeof getCurrentWindow>,
  escala: number,
  intervaloMs: number,
): void {
  zonaArrastre.addEventListener("mousedown", (evento) => {
    if (evento.button !== 0) {
      return;
    }

    detenerPolling();

    void ventana.startDragging();
  });

  document.addEventListener("mouseup", () => {
    // intervaloId !== null: el polling normal sigue vivo, no había
    // arrastre en curso — nada que guardar.
    if (intervaloId !== null) {
      return;
    }

    void guardarPosicionArrastrada(id, ventana, escala, intervaloMs);
  });
}

async function guardarPosicionArrastrada(
  id: string,
  ventana: ReturnType<typeof getCurrentWindow>,
  escala: number,
  intervaloMs: number,
): Promise<void> {
  const posicion = await ventana.outerPosition();

  const destinoX = Math.round(posicion.x + MITAD_ANCHO_PREVIEW_LOGICO * escala);
  const destinoY = Math.round(posicion.y + MITAD_ALTO_PREVIEW_LOGICO * escala);

  await invoke("guardar_posicion_preview_coordenada", {
    id,
    destinoX,
    destinoY,
  }).catch(() => {});

  iniciarPollingPreview(ventana, escala, id, intervaloMs);
}

// ======================================================
// 🏁 INICIAR
// ======================================================

async function iniciar(): Promise<void> {
  const parametros = new URLSearchParams(location.search);
  const idPreview = parametros.get("id");

  if (idPreview !== null) {
    const numero = Number(parametros.get("numero") ?? "0");
    await iniciarPreview(idPreview, numero);
    return;
  }

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

  try {
    const atajo = await conTimeout(
      invoke<AtajoCapturaUI>("obtener_tecla_guardar_coordenada"),
      3000,
      "obtener_tecla_guardar_coordenada",
    );
    teclaGuardar = atajoCapturaATexto(atajo);
  } catch (error) {
    mostrarDiagnostico(
      `FALLÓ obtener_tecla_guardar_coordenada (uso F1): ${String(error)}`,
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
      dibujarLinea(`Presione ${teclaGuardar} para guardar posición`);
      dibujarLinea(`X: ${cursorX}  Y: ${cursorY}`, true);

      if (guardar) {
        guardarYcerrar(cursorX, cursorY);
      }

      break;
    }

    case "relativa_cursor": {
      if (pasoRelativaCursor === 1) {
        dibujarLinea(`Origen: presione ${teclaGuardar} para marcar origen`);
        dibujarLinea(`X: ${cursorX}  Y: ${cursorY}`, true);

        if (guardar) {
          origenRelativaCursor = { x: cursorX, y: cursorY };
          pasoRelativaCursor = 2;
        }
      } else {
        const origen = origenRelativaCursor!;
        const offsetX = cursorX - origen.x;
        const offsetY = cursorY - origen.y;

        dibujarLinea(`Destino: presione ${teclaGuardar} para marcar destino`);
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
        dibujarLinea(`Presione ${teclaGuardar} para guardar posición`);

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
