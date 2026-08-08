// ======================================================
// 🎛️ comp_Controles
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";
import type { FilaPerfil } from "../../core/core_perfil";
import { crearPopup } from "./comp_popup";
import { reconstruirFila } from "../ui_tabla_control";
import { invoke } from "@tauri-apps/api/core";

import {
  abrirPopupTipo,
  abrirPopupColor,
  abrirPopupExtra,
  tipoATexto,
  extraATexto,
} from "./comp_popup_abrir";

import { abrirPopupApp } from "./comp_popup_app";

import {
  abrirPopupExtraTeclaMouse,
  cerrarVentanaCapturaCoordenada,
  textoExtraTeclaMouse,
} from "./comp_popup_coordenada";

import { abrirPopupExtraMultimedia } from "./comp_popup_multimedia_extra";

import { textoExtraMultimedia } from "../../core/core_multimedia";

import { abrirPopupExtraMenuExpress } from "./comp_popup_menu_express_extra";

import { abrirPopupExtraPortapapeles } from "./comp_popup_portapapeles_extra";

import {
  textoMenuExtra,
  crearMenuAccion,
  crearMenuExtra,
} from "../../core/core_menu_express";

import { textoPortapapelesExtra } from "../../core/core_portapapeles";

import { crearCoordenada } from "../../core/core_coordenada";

import { obtenerPerfilUi } from "../../core/core_perfil_ui";

import { filaTieneConflicto } from "../../core/core_conflictos";

// ======================================================
// 🟢🔴 ESTADO (interruptor ON/OFF)
// ======================================================

export function crearEstado(
  contexto: ContextoFila,

  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn estado-toggle";

  const conflicto = filaTieneConflicto(
    filaPerfil.id,

    obtenerPerfilUi().filas,
  );

  boton.dataset.estado = conflicto
    ? "off"
    : filaPerfil.estado === "ON"
      ? "on"
      : "off";

  boton.dataset.conflicto = conflicto ? "true" : "false";

  const texto = document.createElement("span");

  texto.textContent = conflicto ? "OFF" : filaPerfil.estado;

  boton.append(texto);

  if (conflicto) {
    const alerta = document.createElement("span");

    alerta.className = "estado-alerta";

    alerta.textContent = "⚠";

    boton.append(alerta);
  }

  boton.addEventListener(
    "click",

    (evento) => {
      if (conflicto) {
        evento.stopPropagation();

        return;
      }

      filaPerfil.estado = filaPerfil.estado === "ON" ? "OFF" : "ON";

      reconstruirFila(contexto.id);
    },
  );

  return boton;
}

export function crearTipo(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  return crearPopup({
    texto: tipoATexto(filaPerfil.tipo),
    onClick: (evento) => {
      abrirPopupTipo(
        evento,
        (valor) => {
          if (valor === filaPerfil.tipo) {
            return;
          }

          // Si esta fila tenía Coordenada activa y deja de ser
          // tecla_mouse, ya no tiene sentido — no puede quedar una
          // ventana de captura calculando para una fila que ya no
          // puede usar ese extra.
          if (filaPerfil.coordenada.activa && valor !== "tecla_mouse") {
            cerrarVentanaCapturaCoordenada();
          }

          // Al cambiar de Tipo, Acción y Extra dejan de tener sentido
          // para el Tipo anterior: se resetean TODOS los campos de
          // ambas columnas (de todos los tipos) a su valor por
          // defecto, para que no quede guardado ningún dato que ya
          // no está vigente.
          filaPerfil.accion = null;
          filaPerfil.accionReferencia = null;
          filaPerfil.menuAccion = crearMenuAccion();

          filaPerfil.condicion = "Normal";
          filaPerfil.coordenada = crearCoordenada();
          filaPerfil.menuExtra = crearMenuExtra();
          filaPerfil.extra = "normal";

          filaPerfil.tipo = valor;

          reconstruirFila(contexto.id);
        },
        contexto,
      );
    },
  });
}

export function crearExtra(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  // tecla_mouse tiene su propio popup Extra (Simple/Mantenido/Turbo +
  // toggle Coordenada expandible, queda abierto entre selecciones) —
  // totalmente distinto del popup simple que usan los demás tipos.
  if (filaPerfil.tipo === "tecla_mouse") {
    return crearPopup({
      texto: textoExtraTeclaMouse(filaPerfil),
      onClick: (evento) => {
        abrirPopupExtraTeclaMouse(evento, contexto, filaPerfil);
      },
    });
  }

  // multimedia tiene su propio popup Extra (Global/En App) — igual
  // de persistente que el de Tecla/Mouse, pero mucho más chico.
  if (filaPerfil.tipo === "multimedia") {
    return crearPopup({
      texto: textoExtraMultimedia(filaPerfil.extraMultimedia),
      onClick: (evento) => {
        abrirPopupExtraMultimedia(evento, contexto, filaPerfil);
      },
    });
  }

  // menu_express tiene su propio popup Extra (Forma/Limitar
  // cuadrícula/Comportamiento/Ubicación/Tamaños) — igual de
  // persistente que los anteriores.
  if (filaPerfil.tipo === "menu_express") {
    return crearPopup({
      texto: textoMenuExtra(filaPerfil.menuExtra),
      onClick: (evento) => {
        abrirPopupExtraMenuExpress(evento, contexto, filaPerfil);
      },
    });
  }

  // portapapeles tiene su propio popup Extra (Comportamiento/
  // Ubicación/Tamaños/Límite) — igual de persistente que los
  // anteriores.
  if (filaPerfil.tipo === "portapapeles") {
    return crearPopup({
      texto: textoPortapapelesExtra(filaPerfil.portapapelesExtra),
      onClick: (evento) => {
        abrirPopupExtraPortapapeles(evento, contexto, filaPerfil);
      },
    });
  }

  return crearPopup({
    texto: extraATexto(filaPerfil.extra),
    onClick: (evento) => {
      abrirPopupExtra(evento, contexto, filaPerfil);
    },
  });
}

export function crearApp(
  contexto: ContextoFila,

  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn app-control";

  boton.title = filaPerfil.app.programa ?? "Uso global";

  const icono = document.createElement("span");

  icono.className = "app-icono-fallback";

  icono.textContent = filaPerfil.app.programa ? "▣" : "🌐";

  boton.append(icono);

  if (filaPerfil.app.segundoPlano) {
    const indicador = document.createElement("span");

    indicador.className = "app-segundo-plano-indicador";

    indicador.textContent = "∶";

    boton.append(indicador);
  }

  const flecha = document.createElement("span");

  flecha.className = "app-flecha";

  flecha.textContent = "▾";

  boton.append(flecha);

  if (filaPerfil.app.programa) {
    invoke<{
      ancho: number;

      alto: number;

      pixeles: string;
    } | null>(
      "obtener_icono_programa",

      {
        nombre: filaPerfil.app.programa,
      },
    )
      .then((iconoJson) => {
        if (!iconoJson) {
          return;
        }

        const canvas = document.createElement("canvas");

        canvas.width = iconoJson.ancho;

        canvas.height = iconoJson.alto;

        const contextoCanvas = canvas.getContext("2d");

        if (!contextoCanvas) {
          return;
        }

        const pixeles = Uint8ClampedArray.from(
          atob(iconoJson.pixeles),

          (caracter) => caracter.charCodeAt(0),
        );

        contextoCanvas.putImageData(
          new ImageData(
            pixeles,

            iconoJson.ancho,

            iconoJson.alto,
          ),

          0,

          0,
        );

        canvas.className = "app-icono";

        boton.replaceChild(
          canvas,

          icono,
        );
      })

      .catch(() => {});
  }

  boton.addEventListener(
    "click",

    (evento) => {
      abrirPopupApp(
        evento,

        contexto,

        filaPerfil,
      );
    },
  );

  return boton;
}

export function crearColor(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn color-control";
  boton.title = "Color";

  if (filaPerfil.color) {
    const muestra = document.createElement("span");

    muestra.className = "color-control-muestra";
    muestra.style.background = `var(--tag-${filaPerfil.color})`;

    boton.append(muestra);
  } else {
    boton.textContent = "🎨";
  }

  boton.addEventListener("click", (evento) => {
    abrirPopupColor(evento, contexto, filaPerfil);
  });

  return boton;
}

export function crearNota(filaPerfil: FilaPerfil): HTMLDivElement {
  const contenedor = document.createElement("div");
  contenedor.className = "nota-contenedor";

  const input = document.createElement("input");
  input.className = "nota";
  input.placeholder = "Nota...";
  input.value = filaPerfil.nota;

  input.addEventListener("input", () => {
    filaPerfil.nota = input.value;
  });

  const btnEmoji = document.createElement("button");

  btnEmoji.className = "btn-emoji";

  btnEmoji.type = "button";

  btnEmoji.textContent = "\u263A"; // es la carita ☺

  btnEmoji.title = "Insertar emoji";

  btnEmoji.addEventListener("mousedown", async (e) => {
    e.preventDefault();

    await invoke("abrir_selector_emoji");

    input.focus();
  });

  contenedor.appendChild(input);
  contenedor.appendChild(btnEmoji);

  return contenedor;
}
