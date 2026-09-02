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
import {
  getCurrentWindow,
  LogicalSize,
  PhysicalPosition,
} from "@tauri-apps/api/window";


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

  // No-op si no se llegó a marcar un origen (modo relativa_cursor,
  // paso 1) — cerrar_ventana_origen_cursor no falla si la ventana
  // no existe.
  invoke("cerrar_ventana_origen_cursor").catch(() => {});

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
// ------------------------------------------------------
// Confirmación visual: en vez del cartel de texto "✅ Guardado" en
// la posición del card (que no necesariamente coincide con el punto
// capturado en modos relativos/porcentaje), la ventana se reduce a
// un círculo (mismo marcador ⊙ del modo Previsualización) y se
// reposiciona centrada EXACTO sobre el cursor físico en el instante
// del guardado — ese cursor es siempre el punto real capturado,
// más allá de qué valor (absoluto/offset/porcentaje) se haya
// guardado. Se muestra 1 segundo y se cierra sola.
// ======================================================

const LADO_CONFIRMACION_LOGICO = 56;

function guardarYcerrar(
  valorX: number,
  valorY: number,
  cursorFisicoX: number,
  cursorFisicoY: number,
): void {
  detenerPolling();

  invoke("guardar_resultado_coordenada", { x: valorX, y: valorY }).catch(() => {});

  void mostrarFlashConfirmacion(cursorFisicoX, cursorFisicoY);
}

async function mostrarFlashConfirmacion(
  cursorFisicoX: number,
  cursorFisicoY: number,
): Promise<void> {
  const ventana = getCurrentWindow();

  raiz.innerHTML = "";

  const marcador = document.createElement("div");
  marcador.className = "captura-marcador-preview captura-marcador-confirmacion";
  marcador.innerHTML =
    '<svg viewBox="0 0 24 24" width="48" height="48"><circle cx="12" cy="12" r="9" /><circle class="captura-marcador-punto" cx="12" cy="12" r="1.6" /></svg>';
  raiz.append(marcador);

  try {
    const escala = await ventana.scaleFactor();
    const mitadFisica = (LADO_CONFIRMACION_LOGICO / 2) * escala;

    await ventana.setSize(
      new LogicalSize(LADO_CONFIRMACION_LOGICO, LADO_CONFIRMACION_LOGICO),
    );
    await ventana.setPosition(
      new PhysicalPosition(
        cursorFisicoX - mitadFisica,
        cursorFisicoY - mitadFisica,
      ),
    );
  } catch {
    // Sin resize/reposición no queda centrado en el punto exacto,
    // pero igual se muestra y se cierra — mejor que dejar la ventana
    // de captura colgada sin cerrar.
  }

  await new Promise((resolve) => setTimeout(resolve, 1000));

  // Cierra junto con el marcador de origen del modo relativa_cursor
  // (paso 1), si llegó a abrirse — no-op en los demás modos.
  invoke("cerrar_ventana_origen_cursor").catch(() => {});
  invoke("cerrar_ventana_captura_coordenada").catch(() => {});
}

// ======================================================
// 👁️ MODO PREVISUALIZACIÓN (Etapa F)
// ------------------------------------------------------
// Marcador "⊙" en vivo, sin header/Cancelar/texto de
// diagnóstico. El destino ya viene calculado desde Rust
// (obtener_destino_preview_coordenada, por id) — acá solo se
// reposiciona la ventana para centrar el marcador sobre el
// punto. Mitades fijas: la ventana overlay siempre se crea
// con inner_size(56, 56) (ver abrir_ventana_preview_coordenada
// en comandos.rs), no es resizable.
//
// x/y llegan en coordenadas FÍSICAS (mismo sistema que
// GetCursorPos/GetWindowRect en back_coordenada.rs — ver el
// comentario largo sobre esto en back_menu_express.rs::
// monitor_para_punto). Mezclar eso con LogicalPosition rompe el
// posicionamiento en cualquier monitor con escalado != 100%: por
// eso acá se posiciona con PhysicalPosition, convirtiendo la
// mitad de ventana (lógica, 28x28) a físico vía scaleFactor().
//
// El marcador es arrastrable (Regla 17/Bug 5) — ver
// seguirArrastreMarcador() más abajo.
// ======================================================

const MITAD_LADO_PREVIEW_LOGICO = 28;

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

  // Bug 5: config (ubicación/modo/punto de referencia) de ESTA fila
  // — necesaria acá en el frontend para poder calcular el x/y crudo
  // en vivo durante el arrastre (mismas fórmulas que el modo
  // captura normal, ver actualizar()), en vez de depender de
  // startDragging() + outerPosition() al soltar (ver comentario
  // largo en seguirArrastreMarcador: startDragging() nunca deja
  // llegar el mouseup al webview en Windows — Tauri #10767 — por
  // eso antes esto no persistía nunca).
  let config: ConfigCaptura | null = null;

  try {
    const configCruda = await conTimeout(
      invoke<{
        ubicacion: string;
        modo_ventana: string;
        punto_referencia: string;
      } | null>("obtener_config_preview_coordenada", { id }),
      3000,
      "obtener_config_preview_coordenada",
    );

    if (configCruda) {
      config = {
        ubicacion: configCruda.ubicacion as ConfigCaptura["ubicacion"],
        modoVentana: configCruda.modo_ventana as ConfigCaptura["modoVentana"],
        puntoReferencia:
          configCruda.punto_referencia as ConfigCaptura["puntoReferencia"],
      };
    }
  } catch {
    // Sin config no se puede calcular nada en vivo — el arrastre
    // simplemente no se engancha más abajo (zonaArrastre queda sin
    // handlers) y el marcador sigue funcionando en modo solo-lectura
    // (el polling normal de más abajo lo sigue moviendo con el
    // destino ya calculado server-side).
  }

  const ventana = getCurrentWindow();
  const escala = await ventana.scaleFactor();

  // Bug 3: zona de arrastre chica (20px de diámetro, ~10px de radio
  // desde el centro) en vez de todo el marcador (que ocupa la ventana
  // entera) — antes la mano "grab" aparecía en cualquier punto de la
  // ventana, muy lejos del círculo dibujado.
  const zonaArrastre = document.createElement("div");
  zonaArrastre.className = "captura-marcador-zona-arrastre";
  marcador.append(zonaArrastre);

  if (config) {
    activarArrastreMarcador(zonaArrastre, id, config, ventana, escala, intervaloMs);
  }

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
    // Bug 4: antes cualquier error transitorio de IPC (un timeout
    // puntual, etc.) cortaba el polling para siempre con
    // detenerPolling() — el marcador quedaba "congelado" en su
    // última posición en vez de seguir a la ventana/cursor. Ahora se
    // salta este tick nada más; el intervalo sigue vivo y lo
    // reintenta en el próximo (100ms después por defecto).
    return;
  }

  if (!destino) {
    return;
  }

  const [x, y] = destino;

  await ventana.setPosition(
    new PhysicalPosition(
      x - MITAD_LADO_PREVIEW_LOGICO * escala,
      y - MITAD_LADO_PREVIEW_LOGICO * escala,
    ),
  );
}

// ======================================================
// 🖱️ ARRASTRAR EL MARCADOR (Regla 17 / Bug 5)
// ------------------------------------------------------
// Antes esto usaba ventana.startDragging() (arrastre nativo del
// SO) + esperar mouseup en document para recién ahí leer
// outerPosition() y persistir. Eso NUNCA funcionó de forma
// confiable en Windows: es un bug conocido de Tauri/Tao (issue
// #10767) que el evento mouseup jamás llega al webview de origen
// tras un startDragging() — así que guardarPosicionArrastrada()
// nunca se ejecutaba y el marcador "volvía a su lugar" al reabrir.
//
// Ahora, en vez de delegarle el movimiento al SO, esta ventana
// escucha mousemove ella misma mientras el botón sigue apretado
// (modo captura "encubierto", mismo cálculo que actualizar() para
// el modo captura normal) y:
//   1. calcula el x/y CRUDO en vivo con puntoReferenciaAbsoluto()
//      según ubicacion/modoVentana/puntoReferencia de esta fila,
//   2. lo escribe en memoria (actualizar_xy_preview_en_vivo, SIN
//      tocar disco) para que el polling de la fila en el Gestor
//      (cada 300ms) lo refleje en vivo en la columna X,Y,
//   3. mueve la ventana overlay al punto de pantalla resultante
//      con setPosition, para que el marcador visual siga al mouse.
// Al soltar (mouseup, escuchado en ESTA ventana — no en document
// global, que es justamente lo que fallaba con startDragging) se
// persiste a disco con guardar_posicion_preview_coordenada.
// ======================================================

function activarArrastreMarcador(
  zonaArrastre: HTMLElement,
  id: string,
  config: ConfigCaptura,
  ventana: ReturnType<typeof getCurrentWindow>,
  escala: number,
  intervaloMs: number,
): void {
  let arrastrando = false;
  let ultimoXY: { x: number; y: number } | null = null;

  const calcularXYCrudo = (
    cursorX: number,
    cursorY: number,
    ventanaActiva: VentanaActiva | null,
  ): { x: number; y: number } | null => {
    switch (config.ubicacion) {
      case "absoluta":
        return { x: cursorX, y: cursorY };

      case "relativa_cursor":
        // Pendiente (Bug 5, a definir): no hay un "origen" fijo
        // durante el arrastre de un marcador ya existente (a
        // diferencia del modo captura de 2 pasos, que sí tiene un
        // punto de origen marcado explícitamente por el usuario).
        // Por ahora este tipo NO se persiste al arrastrar — null
        // hace que onMouseMove no actualice nada (ver más abajo) y
        // el marcador se puede mover visualmente pero suelta sin
        // guardar.
        return null;

      case "relativa_ventana": {
        if (!ventanaActiva) {
          return null;
        }

        if (config.modoVentana === "porcentaje") {
          const h = ((cursorX - ventanaActiva.x) / ventanaActiva.ancho) * 100;
          const v = ((cursorY - ventanaActiva.y) / ventanaActiva.alto) * 100;
          return { x: h, y: v };
        }

        const base = puntoReferenciaAbsoluto(config.puntoReferencia, ventanaActiva);
        return { x: cursorX - base.x, y: cursorY - base.y };
      }
    }
  };

  const onMouseMove = async (evento: MouseEvent): Promise<void> => {
    if (!arrastrando) {
      return;
    }

    let cursor: [number, number] | null;
    let ventanaActiva: VentanaActiva | null;

    try {
      [cursor, ventanaActiva] = await Promise.all([
        invoke<[number, number]>("obtener_cursor_captura"),
        invoke<VentanaActiva | null>("obtener_ventana_activa_captura"),
      ]);
    } catch {
      return;
    }

    if (!cursor) {
      return;
    }

    const [cursorX, cursorY] = cursor;

    const xy = calcularXYCrudo(cursorX, cursorY, ventanaActiva);

    if (!xy) {
      return;
    }

    ultimoXY = xy;

    invoke("actualizar_xy_preview_en_vivo", { id, x: xy.x, y: xy.y }).catch(
      () => {},
    );

    await ventana.setPosition(
      new PhysicalPosition(
        cursorX - MITAD_LADO_PREVIEW_LOGICO * escala,
        cursorY - MITAD_LADO_PREVIEW_LOGICO * escala,
      ),
    );

    evento.preventDefault();
  };

  const onMouseUp = (): void => {
    if (!arrastrando) {
      return;
    }

    arrastrando = false;
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);

    if (ultimoXY) {
      void invoke("guardar_posicion_preview_coordenada", {
        id,
        x: ultimoXY.x,
        y: ultimoXY.y,
      }).catch(() => {});
    }

    iniciarPollingPreview(ventana, escala, id, intervaloMs);
  };

  zonaArrastre.addEventListener("mousedown", (evento) => {
    if (evento.button !== 0) {
      return;
    }

    detenerPolling();

    arrastrando = true;
    ultimoXY = null;

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  });
}

// ======================================================
// 📍 MARCADOR ESTÁTICO DE ORIGEN (modo "Relativa a cursor")
// ------------------------------------------------------
// Ventana propia (abrir_ventana_origen_cursor en comandos.rs) ya
// creada centrada y del tamaño final — acá solo dibuja el círculo,
// sin polling ni arrastre: es un punto fijo que marca dónde se
// presionó el primer F1, hasta que el segundo F1 cierra esta
// ventana junto con la de captura (ver cerrar_ventana_origen_cursor
// llamado desde mostrarFlashConfirmacion/cerrar en este archivo).
// ======================================================

function iniciarOrigenCursor(): void {
  raiz.innerHTML = "";

  const marcador = document.createElement("div");
  marcador.className = "captura-marcador-preview";
  marcador.innerHTML =
    '<svg viewBox="0 0 24 24" width="48" height="48"><circle cx="12" cy="12" r="9" /><circle class="captura-marcador-punto" cx="12" cy="12" r="1.6" /></svg>';
  raiz.append(marcador);
}

// ======================================================
// 🏁 INICIAR
// ======================================================

async function iniciar(): Promise<void> {
  const parametros = new URLSearchParams(location.search);
  const idPreview = parametros.get("id");
  const modo = parametros.get("modo");

  if (idPreview !== null) {
    const numero = Number(parametros.get("numero") ?? "0");
    await iniciarPreview(idPreview, numero);
    return;
  }

  if (modo === "origen_cursor") {
    iniciarOrigenCursor();
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
        guardarYcerrar(cursorX, cursorY, cursorX, cursorY);
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

          invoke("abrir_ventana_origen_cursor", {
            x: cursorX,
            y: cursorY,
          }).catch(() => {});
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
          guardarYcerrar(offsetX, offsetY, cursorX, cursorY);
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
          guardarYcerrar(h, v, cursorX, cursorY);
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
          guardarYcerrar(offsetX, offsetY, cursorX, cursorY);
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
