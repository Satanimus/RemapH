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
// ======================================================

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";

import { aplicarOverridesApariencia } from "../core/core_apariencia";
import type { EstadoGrabacionMacro } from "../core/core_grabacion_macro";

import "../styles/styl_variables.css";
import "../styles/styl_indicador_macro.css";

void aplicarOverridesApariencia();

type ModoIndicadorMacro = "grabacion" | "play";

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

  setInterval(() => {
    invoke<ProgresoIndicadorMacro>("obtener_progreso_indicador_macro")
      .then((progreso) => {
        texto.textContent = textoProgresoPlay(progreso);
      })
      .catch(() => {
        // Ventana en cierre — nada que hacer.
      });
  }, 200);
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

  const tecla = parametros.get("tecla") ?? "";
  iniciarModoGrabacion(raiz, tecla);
}

iniciar();
