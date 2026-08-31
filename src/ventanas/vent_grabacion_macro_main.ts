// ======================================================
// 🔴 vent_grabacion_macro_Main
// ------------------------------------------------------
// Punto de entrada de la ventana overlay del indicador de
// grabación de Macro (grabacion_macro.html — página
// independiente, ver vite.config.ts). El nombre de la tecla
// toggle llega una sola vez por query param (?tecla=...),
// fijado por comandos.rs al crear la ventana — eso no cambia
// en toda la vida de la ventana. El ESTADO (🟡 armada / 🔴
// activa) sí cambia con la tecla física, así que esta ventana
// hace su propio polling corto sobre
// obtener_estado_grabacion_macro (mismo patrón que el editor,
// ver comp_popup_macro_editor.ts) para reflejarlo en vivo —
// no espera ningún invoke del editor.
// ======================================================

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";

import { aplicarOverridesApariencia } from "../core/core_apariencia";
import type { EstadoGrabacionMacro } from "../core/core_grabacion_macro";

import "../styles/styl_variables.css";
import "../styles/styl_grabacion_macro.css";

void aplicarOverridesApariencia();

function textoEstado(tecla: string, estado: EstadoGrabacionMacro): string {
  if (estado === "activa") {
    return `Presione ${tecla} para detener`;
  }

  return `Presione ${tecla} para grabar`;
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

function iniciar(): void {
  const raiz = document.getElementById("grabacion");
  if (!raiz) return;

  activarArrastre(raiz);

  const parametros = new URLSearchParams(window.location.search);
  const tecla = parametros.get("tecla") ?? "";

  const punto = document.createElement("span");
  punto.className = "grabacion-punto";
  punto.dataset.estado = "armada";

  const texto = document.createElement("span");
  texto.className = "grabacion-texto";
  texto.textContent = textoEstado(tecla, "armada");

  raiz.append(punto, texto);

  let estadoActual: EstadoGrabacionMacro = "armada";

  setInterval(() => {
    invoke<EstadoGrabacionMacro>("obtener_estado_grabacion_macro")
      .then((nuevoEstado) => {
        if (nuevoEstado === estadoActual || nuevoEstado === "inactiva") {
          // "inactiva" significa que la ventana ya está por cerrarse
          // (cerrar_ventana_grabacion_macro, disparado desde el
          // editor al detectar Activa→Inactiva) — no vale la pena
          // repintar el instante previo al cierre.
          return;
        }

        estadoActual = nuevoEstado;
        punto.dataset.estado = nuevoEstado;
        texto.textContent = textoEstado(tecla, nuevoEstado);
      })
      .catch(() => {
        // Ventana huérfana/en cierre — nada que hacer.
      });
  }, 200);
}

iniciar();
