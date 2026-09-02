// ======================================================
// 🟢🔴 vent_Indicador_Macro_Main
// ------------------------------------------------------
// Punto de entrada de la ventana overlay Indicador_Macro
// (indicador_macro.html — página independiente, ver
// vite.config.ts). Reemplaza a vent_grabacion_macro_main.ts:
// misma ventana/label, ahora con dos modos.
//
// Modo "grabacion": el nombre de la tecla toggle llega una
// sola vez por query param (?modo=grabacion&tecla=...),
// fijado por comandos.rs al crear la ventana — eso no
// cambia en toda la vida de la ventana. El ESTADO (🟡 armada
// / 🔴 activa) sí cambia con la tecla física, así que esta
// ventana hace su propio polling corto sobre
// obtener_estado_grabacion_macro (mismo patrón que el
// editor, ver comp_popup_macro_editor.ts) para reflejarlo en
// vivo — no espera ningún invoke del editor.
//
// Modo "play" (?modo=play): punto verde fijo + contador
// "paso_actual / total_pasos". El progreso se consulta por
// polling corto sobre obtener_progreso_indicador_macro
// (runt_macro.rs).
//
// Modo "ubicar" (?modo=ubicar): texto fijo "Arrastrame", sin
// polling — se abre/cierra desde el popup Extra de Macro
// (comp_popup_macro_extra.ts) para reposicionar la ventana a mano
// cuando no hay una ejecución real en curso (en Play el mouse está
// en movimiento y no es viable arrastrarla ahí).
// ======================================================

import { invoke } from "@tauri-apps/api/core";
import {
  getCurrentWindow,
  LogicalSize,
  PhysicalPosition,
} from "@tauri-apps/api/window";

import { aplicarOverridesApariencia } from "../core/core_apariencia";
import type { EstadoGrabacionMacro } from "../core/core_grabacion_macro";

import "../styles/styl_variables.css";
import "../styles/styl_indicador_macro.css";

void aplicarOverridesApariencia();

// Espejo de ALTO_INDICADOR_MACRO_LOGICO en comandos.rs — el ancho se
// ajusta al contenido (ver ajustarAnchoAlContenido), el alto queda
// fijo.
const ALTO_INDICADOR_MACRO_LOGICO = 40;

type ModoIndicadorMacro = "grabacion" | "play" | "ubicar";

interface ProgresoIndicadorMacro {
  pasoActual: number;
  totalPasos: number;
}

function textoEstadoGrabacion(
  tecla: string,
  estado: EstadoGrabacionMacro,
): string {
  if (estado === "activa") {
    return `Presione ${tecla} para detener`;
  }

  return `Presione ${tecla} para grabar`;
}

function textoProgresoPlay(progreso: ProgresoIndicadorMacro): string {
  const pasoActual = String(progreso.pasoActual).padStart(2, "0");
  const totalPasos = String(progreso.totalPasos).padStart(2, "0");

  return `${pasoActual} / ${totalPasos}`;
}

// ======================================================
// 🖱️ ARRASTRE MANUAL
// ------------------------------------------------------
// NO se usa data-tauri-drag-region / startDragging() nativo: mismo
// bug de Tauri/Tao ya documentado en vent_captura_main.ts (issue
// #10767) — en Windows el arrastre nativo no es confiable para
// estas ventanas overlay.
//
// El intento anterior calculaba el delta con evento.screenX/screenY
// (coordenadas del evento del mouse dentro del webview) multiplicado
// por scaleFactor() — en Webview2/Windows esas coordenadas no
// siempre están en la misma base física que outerPosition(), lo que
// hacía que el arrastre no funcionara. Se cambia al mismo patrón que
// SÍ funciona en el marcador arrastrable de vent_captura_main.ts
// (activarArrastreMarcador): en cada mousemove se pide el cursor
// físico real vía el comando obtener_cursor_captura (mismo backend,
// GetCursorPos) y la ventana se reposiciona directo a esa coordenada
// menos el offset fijado al mousedown — sin depender de deltas de
// eventos del webview.
// ======================================================

function activarArrastre(raiz: HTMLElement): void {
  const ventana = getCurrentWindow();

  let arrastrando = false;
  let offsetX = 0;
  let offsetY = 0;

  const alMover = async (): Promise<void> => {
    if (!arrastrando) return;

    let cursor: [number, number];

    try {
      cursor = await invoke<[number, number]>("obtener_cursor_captura");
    } catch {
      return;
    }

    if (!arrastrando) return; // pudo soltarse mientras el invoke estaba en vuelo.

    const [cursorX, cursorY] = cursor;

    void ventana.setPosition(
      new PhysicalPosition(cursorX - offsetX, cursorY - offsetY),
    );
  };

  const alSoltar = (): void => {
    arrastrando = false;

    document.removeEventListener("mousemove", alMover);
    document.removeEventListener("mouseup", alSoltar);

    // Última posición: se persiste al soltar.
    void guardarPosicionTrasArrastre(ventana);
  };

  raiz.addEventListener("mousedown", (evento) => {
    if (evento.button !== 0) return;

    void (async () => {
      const [posicion, cursor] = await Promise.all([
        ventana.outerPosition(),
        invoke<[number, number]>("obtener_cursor_captura"),
      ]);

      offsetX = cursor[0] - posicion.x;
      offsetY = cursor[1] - posicion.y;
      arrastrando = true;

      document.addEventListener("mousemove", alMover);
      document.addEventListener("mouseup", alSoltar);
    })();
  });
}

// Persiste la posición actual de la ventana (Etapa C:
// guardar_posicion_indicador_macro) en coordenadas LÓGICAS — mismo
// sistema que usa comandos.rs al crear/posicionar la ventana
// (posicion_x/posicion_y ahí están divididas por scale_factor()).
// outerPosition() devuelve físicas, así que hay que escalar antes
// de guardar o la posición restaurada no coincidiría con la
// arrastrada.
async function guardarPosicionTrasArrastre(
  ventana: ReturnType<typeof getCurrentWindow>,
): Promise<void> {
  try {
    const [posicion, escala] = await Promise.all([
      ventana.outerPosition(),
      ventana.scaleFactor(),
    ]);

    await invoke("guardar_posicion_indicador_macro", {
      x: posicion.x / escala,
      y: posicion.y / escala,
    });
  } catch (error) {
    console.error("❌ No se pudo guardar la posición del indicador:", error);
  }
}

// ======================================================
// 📏 ANCHO AL CONTENIDO
// ------------------------------------------------------
// La ventana nace con un ancho fijo (comandos.rs, solo para el
// primer instante antes de que haya contenido que medir). Acá se
// ajusta al ancho real del contenido (punto + texto + padding) cada
// vez que el texto cambia, para que "🟢 03 / 15" no quede tan ancho
// como "Presione Control Izquierdo + F1 para grabar" ni viceversa.
// raiz tiene width:100% por CSS (llena la ventana) — se fuerza a
// max-content un instante para medir su ancho natural y se revierte.
// ======================================================

function ajustarAnchoAlContenido(raiz: HTMLElement): void {
  raiz.style.width = "max-content";
  const ancho = Math.ceil(raiz.getBoundingClientRect().width);
  raiz.style.width = "";

  void getCurrentWindow()
    .setSize(new LogicalSize(ancho, ALTO_INDICADOR_MACRO_LOGICO))
    .catch(() => {
      // Ventana en cierre — nada que hacer.
    });
}

// ======================================================
// 🔴 MODO GRABACIÓN
// ======================================================

function iniciarModoGrabacion(raiz: HTMLElement, tecla: string): void {
  const punto = document.createElement("span");
  punto.className = "indicador-macro-punto";
  punto.dataset.estado = "armada";

  const texto = document.createElement("span");
  texto.className = "indicador-macro-texto";
  texto.textContent = textoEstadoGrabacion(tecla, "armada");

  raiz.append(punto, texto);
  ajustarAnchoAlContenido(raiz);

  let estadoActual: EstadoGrabacionMacro = "armada";

  setInterval(() => {
    invoke<EstadoGrabacionMacro>("obtener_estado_grabacion_macro")
      .then((nuevoEstado) => {
        if (nuevoEstado === estadoActual || nuevoEstado === "inactiva") {
          // "inactiva" significa que la ventana ya está por cerrarse
          // (cerrar_ventana_indicador_macro, disparado desde el
          // editor al detectar Activa→Inactiva) — no vale la pena
          // repintar el instante previo al cierre.
          return;
        }

        estadoActual = nuevoEstado;
        punto.dataset.estado = nuevoEstado;
        texto.textContent = textoEstadoGrabacion(tecla, nuevoEstado);
        ajustarAnchoAlContenido(raiz);
      })
      .catch(() => {
        // Ventana huérfana/en cierre — nada que hacer.
      });
  }, 200);
}

// ======================================================
// 🟢 MODO PLAY
// ------------------------------------------------------
// El catch silencioso deja el contador sin actualizar ante un
// fallo — mismo criterio de tolerancia a fallos que el polling
// de modo Grabación.
// ======================================================

function iniciarModoPlay(raiz: HTMLElement): void {
  const punto = document.createElement("span");
  punto.className = "indicador-macro-punto";
  punto.dataset.estado = "play";

  const texto = document.createElement("span");
  texto.className = "indicador-macro-texto";
  texto.textContent = "00 / 00";

  raiz.append(punto, texto);
  ajustarAnchoAlContenido(raiz);

  setInterval(() => {
    invoke<ProgresoIndicadorMacro>("obtener_progreso_indicador_macro")
      .then((progreso) => {
        texto.textContent = textoProgresoPlay(progreso);
        ajustarAnchoAlContenido(raiz);
      })
      .catch(() => {
        // Ventana en cierre — nada que hacer.
      });
  }, 200);
}

// ======================================================
// 📍 MODO UBICAR
// ------------------------------------------------------
// Texto fijo, sin polling — el arrastre y el guardado de posición
// son los mismos de siempre (activarArrastre/guardarPosicionTrasArrastre).
// ======================================================

function iniciarModoUbicar(raiz: HTMLElement): void {
  const punto = document.createElement("span");
  punto.className = "indicador-macro-punto";
  punto.dataset.estado = "play";

  const texto = document.createElement("span");
  texto.className = "indicador-macro-texto";
  texto.textContent = "Arrastrame";

  raiz.append(punto, texto);
  ajustarAnchoAlContenido(raiz);
}

function iniciar(): void {
  const raiz = document.getElementById("indicador-macro");
  if (!raiz) return;

  activarArrastre(raiz);

  const parametros = new URLSearchParams(window.location.search);
  const modo = (parametros.get("modo") ?? "grabacion") as ModoIndicadorMacro;

  if (modo === "play") {
    iniciarModoPlay(raiz);
    return;
  }

  if (modo === "ubicar") {
    iniciarModoUbicar(raiz);
    return;
  }

  const tecla = parametros.get("tecla") ?? "";
  iniciarModoGrabacion(raiz, tecla);
}

iniciar();
