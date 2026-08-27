// ======================================================
// 🖥️ comp_Popup_App
// ------------------------------------------------------
// Popup de selección de programa. Estilo alineado con el
// popup Extra de Tecla/Mouse (comp_popup_coordenada.ts):
// popup persistente que se redibuja en el lugar en vez de
// cerrarse en cada interacción, interruptor deslizante y
// caja oscura para el listado.
//
// Uso global
// Segundo plano (interruptor — no cierra el popup)
// ───────────
// Listado de programas:  [Principales] [Otros]
// ┌ caja oscura ─────────────┐
// │ Programas (según filtro) │
// └───────────────────────────┘
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import { crearInterruptor, crearGrupoOpciones } from "./comp_popup_grupo";

// ======================================================
// 📦 MODELOS BACKEND
// ======================================================

interface IconoJson {
  ancho: number;

  alto: number;

  pixeles: string;
}

interface ProcesoIconoJson {
  nombre: string;

  icono: IconoJson | null;
}

// ======================================================
// 🎨 ICONO FALLBACK
// ======================================================

function crearIconoFallback(): HTMLElement {
  const icono = document.createElement("span");

  icono.className = "app-icono-fallback";

  icono.textContent = "▣";

  return icono;
}

// ======================================================
// 🖼️ CREAR ICONO REAL
// ======================================================

function crearIcono(datos: IconoJson): HTMLElement {
  const canvas = document.createElement("canvas");

  canvas.width = datos.ancho;

  canvas.height = datos.alto;

  const contexto = canvas.getContext("2d");

  if (!contexto) {
    return crearIconoFallback();
  }

  const pixeles = Uint8ClampedArray.from(
    atob(datos.pixeles),

    (caracter) => caracter.charCodeAt(0),
  );

  const imagen = new ImageData(
    pixeles,

    datos.ancho,

    datos.alto,
  );

  contexto.putImageData(
    imagen,

    0,

    0,
  );

  canvas.className = "app-icono";

  return canvas;
}

// ======================================================
// 🧩 ICONO DE PROCESO
// ======================================================

function crearIconoProceso(proceso: ProcesoIconoJson): HTMLElement {
  if (!proceso.icono) {
    return crearIconoFallback();
  }

  return crearIcono(proceso.icono);
}

// ======================================================
// 🔘 BOTÓN DE PROCESO
// ======================================================

function crearBotonProceso(
  proceso: ProcesoIconoJson,

  seleccionar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn app-popup-programa";

  boton.append(crearIconoProceso(proceso));

  const nombre = document.createElement("span");

  nombre.className = "app-popup-nombre";

  nombre.textContent = proceso.nombre;

  boton.append(nombre);

  boton.addEventListener(
    "click",

    () => {
      seleccionar();

      ocultarPopup();
    },
  );

  return boton;
}

// ======================================================
// 🌐 USO GLOBAL
// ======================================================

function crearBotonGlobal(
  filaPerfil: FilaPerfil,

  contexto: ContextoFila,

  alModificar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn app-popup-global";

  const icono = document.createElement("span");

  icono.className = "app-popup-global-icono";

  icono.textContent = "🌐";

  boton.append(icono);

  const texto = document.createElement("span");

  texto.textContent = "Uso global";

  boton.append(texto);

  boton.addEventListener(
    "click",

    () => {
      filaPerfil.app.programa = null;

      // Si esta fila era Multimedia con alcance "En App", vaciar la
      // columna App le quita el programa al que apuntaba — se
      // resetea sola a "global" (regla acordada, ver
      // comp_popup_multimedia_extra.ts).
      if (filaPerfil.extraMultimedia === "en_app") {
        filaPerfil.extraMultimedia = "global";
      }

      reconstruirFila(contexto.id);

      alModificar();

      ocultarPopup();
    },
  );

  return boton;
}

// ======================================================
// 🟢 SEGUNDO PLANO
// ------------------------------------------------------
// Mismo interruptor deslizante que usa Coordenada en el
// popup Extra (comp_popup_grupo.ts): gris oscuro apagado,
// bolita en vez de checkbox. Togglear NO cierra el popup —
// solo reconstruye la fila y redibuja el mismo popup en el
// lugar, igual que el resto de los controles persistentes.
// ======================================================

function crearSegundoPlano(
  filaPerfil: FilaPerfil,

  contexto: ContextoFila,

  alModificar: () => void,

  redibujar: () => void,
): HTMLElement {
  return crearInterruptor(
    "Segundo plano :",

    filaPerfil.app.segundoPlano,

    () => {
      filaPerfil.app.segundoPlano = !filaPerfil.app.segundoPlano;

      reconstruirFila(contexto.id);

      alModificar();

      redibujar();
    },
  );
}

// ======================================================
// ➖ SEPARADOR
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

// ======================================================
// 📋 LISTADO DE PROGRAMAS (subtítulo + Principales/Otros + caja)
// ------------------------------------------------------
// Mismo patrón que el popup Extra de Tecla/Mouse: subtítulo
// con el mismo estilo que "Ubicación relativa a:" (clase
// popup-fila-label, vía app-popup-lista-titulo) + grupo de
// opciones tipo-radio (Principales/Otros) que decide qué
// lista se muestra, todo dentro de una caja oscura
// (popup-caja-interna) tomada del popup de Coordenada. Elegir
// Principales/Otros o un programa NO cierra el popup — salvo
// elegir un programa puntual, que sí selecciona y cierra
// (mismo comportamiento que antes tenían los botones de
// programa).
// ======================================================

type FiltroListado = "principales" | "otros";

const LISTADO_OPCIONES: { texto: string; valor: FiltroListado }[] = [
  { texto: "Principales", valor: "principales" },
  { texto: "Otros", valor: "otros" },
];

function crearListadoProgramas(
  procesos: ProcesoIconoJson[],

  contexto: ContextoFila,

  filaPerfil: FilaPerfil,

  alModificar: () => void,

  filtro: FiltroListado,

  onCambiarFiltro: (filtro: FiltroListado) => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  contenedor.className = "popup-fila";

  const titulo = document.createElement("span");

  titulo.className = "app-popup-lista-titulo";

  titulo.textContent = "Listado de programas:";

  contenedor.append(titulo);

  contenedor.append(
    crearGrupoOpciones(LISTADO_OPCIONES, filtro, (valor) => {
      onCambiarFiltro(valor);
    }),
  );

  const caja = document.createElement("div");

  caja.className = "popup-caja-interna app-popup-lista-caja";

  const lista = document.createElement("div");

  lista.className = "app-popup-lista";

  procesos

    .filter((proceso) =>
      filtro === "principales" ? proceso.icono !== null : !proceso.icono,
    )

    .forEach((proceso) => {
      lista.append(
        crearBotonProceso(
          proceso,

          () => {
            filaPerfil.app.programa = proceso.nombre;

            reconstruirFila(contexto.id);

            alModificar();
          },
        ),
      );
    });

  caja.append(lista);

  contenedor.append(caja);

  return contenedor;
}

// ======================================================
// 🖥️ ABRIR POPUP APP
// ======================================================

export async function abrirPopupApp(
  evento: MouseEvent,

  contexto: ContextoFila,

  filaPerfil: FilaPerfil,

  alModificar: () => void,

  filtroInicial: FiltroListado = "principales",
): Promise<void> {
  const procesos = await invoke<ProcesoIconoJson[]>("listar_procesos_ventana");

  const popup = document.createElement("div");

  popup.className = "app-popup";

  popup.dataset.ayudaId = "popup-app";

  const redibujar = (filtro: FiltroListado) =>
    abrirPopupApp(evento, contexto, filaPerfil, alModificar, filtro);

  popup.append(
    crearBotonGlobal(filaPerfil, contexto, alModificar),

    crearSegundoPlano(filaPerfil, contexto, alModificar, () =>
      redibujar(filtroInicial),
    ),

    crearSeparador(),
  );

  popup.append(
    crearListadoProgramas(
      procesos,

      contexto,

      filaPerfil,

      alModificar,

      filtroInicial,

      (filtro) => redibujar(filtro),
    ),
  );

  mostrarPopup(
    popup,

    evento.clientX,

    evento.clientY,
  );
}
