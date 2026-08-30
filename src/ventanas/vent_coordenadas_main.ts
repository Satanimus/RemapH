// ======================================================
// 📍 vent_Coordenadas_Main
// ------------------------------------------------------
// Punto de entrada de la ventana "Gestor de Coordenadas
// guardadas" (coordenadas.html — página independiente, ver
// vite.config.ts). Ventana normal decorada (no overlay).
//
// Lista + filtra el catálogo (banco_coordenadas.rs vía
// core_banco_coordenadas.ts) y permite editar/eliminar cada
// fila. La tabla de filas (columnas #/⁝/⊙️/▶/X/Grupo/Nombre/
// Tipo/X,Y) se termina de portar en las etapas siguientes.
// ======================================================

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { aplicarOverridesApariencia } from "../core/core_apariencia";
import { crearControladorArrastre } from "../util/util_arrastrable";
import {
  mostrarPopup,
  crearContenedorPopup,
  ocultarPopup,
  actualizarContenidoPopup,
} from "../componentes/comp_popup_contenedor";
import {
  crearGrupoOpciones,
  crearFilaPopup,
} from "../componentes/comp_popup_grupo";

import {
  type CoordenadaBanco,
  type CoordenadaBancoJson,
  convertirCoordenadaBanco,
  coordenadaBancoParaBackend,
  crearCoordenadaBanco,
  textoTipoCoordenada,
  textoModoCoordenada,
  textoPuntoReferenciaCoordenada,
  TIPO_A_UBICACION,
  MODO_A_MODO_VENTANA,
  PUNTO_REFERENCIA_NUMERO_A_STRING,
} from "../core/core_banco_coordenadas";

import "../styles/styl_variables.css";
import "../styles/styl_general.css";
import "../styles/styl_botones.css";
import "../styles/styl_arrastrable.css";
import "../styles/styl_layout.css";
import "../styles/styl_coordenadas.css";

void aplicarOverridesApariencia();

// ======================================================
// 🏗️ ARMAR DOM
// ======================================================

const raiz = document.getElementById("coordenadas")!;

const card = document.createElement("div");
card.className = "coordenadas-card";

// --- Barra superior (sin título de texto, solo botones) ---

const barraSuperior = document.createElement("div");
barraSuperior.className = "coordenadas-barra-superior";

const botonAgregarFila = document.createElement("button");
botonAgregarFila.type = "button";
botonAgregarFila.className = "coordenadas-boton coordenadas-boton-primario";
botonAgregarFila.textContent = "+ Fila coordenada";
botonAgregarFila.addEventListener("click", () => {
  void agregarFila();
});

const botonFijar = document.createElement("button");
botonFijar.type = "button";
botonFijar.className = "btn-ayuda coordenadas-btn-fijar";
botonFijar.textContent = "📌";
botonFijar.title = "Fijar ventana";

barraSuperior.append(botonAgregarFila, botonFijar);

// --- Tabla ---

const tabla = document.createElement("table");
tabla.className = "coordenadas-tabla";

const thead = document.createElement("thead");
const filaEncabezado = document.createElement("tr");

function crearEncabezado(
  contenido: string | HTMLElement,
): HTMLTableCellElement {
  const th = document.createElement("th");
  th.append(contenido);
  return th;
}

// ======================================================
// 🔽 FILTROS DE ENCABEZADO (Grupo / Tipo) — popup en vez de
// <select> nativo, para tener control total del tema oscuro
// (el listado desplegable de <select> usa el estilo del SO).
// ======================================================

let filtroGrupo = "";
let filtroTipo = "";
let gruposDisponibles: string[] = [];

const botonFiltroGrupo = document.createElement("button");
botonFiltroGrupo.type = "button";
botonFiltroGrupo.className = "coordenadas-boton-filtro";

const botonFiltroTipo = document.createElement("button");
botonFiltroTipo.type = "button";
botonFiltroTipo.className = "coordenadas-boton-filtro";

function actualizarTextoFiltros(): void {
  botonFiltroGrupo.textContent = `Grupo: ${filtroGrupo || "Todos"}`;
  botonFiltroTipo.textContent = `Tipo: ${filtroTipo ? textoTipoCoordenada(Number(filtroTipo)) : "Todos"}`;

  // Regla 5: borde cyan mientras el filtro esté activo.
  botonFiltroGrupo.classList.toggle(
    "coordenadas-boton-filtro-activo",
    filtroGrupo !== "",
  );
  botonFiltroTipo.classList.toggle(
    "coordenadas-boton-filtro-activo",
    filtroTipo !== "",
  );
}

actualizarTextoFiltros();

botonFiltroGrupo.addEventListener("click", (evento) => {
  void cargarGruposFiltro().then(() => {
    const lista = document.createElement("div");
    lista.className = "popup-lista";

    lista.append(
      crearGrupoOpciones(
        [
          { texto: "Todos", valor: "" },
          ...gruposDisponibles.map((grupo) => ({ texto: grupo, valor: grupo })),
        ],
        filtroGrupo,
        (valor) => {
          filtroGrupo = valor;
          actualizarTextoFiltros();
          recargarPorFiltro();
          ocultarPopup();
        },
        "popup-grupo-vertical",
      ),
    );

    mostrarPopup(lista, evento.clientX, evento.clientY);
  });
});

botonFiltroTipo.addEventListener("click", (evento) => {
  const lista = document.createElement("div");
  lista.className = "popup-lista";

  lista.append(
    crearGrupoOpciones(
      [
        { texto: "Todos", valor: "" },
        { texto: "Absoluta", valor: "1" },
        { texto: "Cursor", valor: "2" },
        { texto: "Ventana", valor: "3" },
      ],
      filtroTipo,
      (valor) => {
        filtroTipo = valor;
        actualizarTextoFiltros();
        recargarPorFiltro();
        ocultarPopup();
      },
      "popup-grupo-vertical",
    ),
  );

  mostrarPopup(lista, evento.clientX, evento.clientY);
});

const botonAsaEncabezado = document.createElement("button");
botonAsaEncabezado.type = "button";
botonAsaEncabezado.className = "coordenadas-asa";
botonAsaEncabezado.textContent = "⁝";
botonAsaEncabezado.title = "Mostrar/ocultar opciones";
botonAsaEncabezado.addEventListener("click", () => {
  tabla.classList.toggle("coordenadas-tabla--iconos-desplegados");
});

const botonPreviewGlobal = document.createElement("button");
botonPreviewGlobal.type = "button";
botonPreviewGlobal.className =
  "coordenadas-boton-icono coordenadas-icono-togglable";
botonPreviewGlobal.textContent = "⊙";
botonPreviewGlobal.title = "Previsualizar todas";
botonPreviewGlobal.addEventListener("click", () => {
  void alternarPrevisualizacionGlobal();
});

const celdaOpcionesEncabezado = document.createElement("div");
celdaOpcionesEncabezado.className = "coordenadas-celda-opciones";
celdaOpcionesEncabezado.append(botonAsaEncabezado, botonPreviewGlobal);

const thNumero = crearEncabezado("✓");
thNumero.title = "Seleccionar esta fila";

filaEncabezado.append(
  thNumero,
  crearEncabezado(celdaOpcionesEncabezado),
  crearEncabezado(botonFiltroGrupo),
  crearEncabezado("Nombre"),
  crearEncabezado(botonFiltroTipo),
  crearEncabezado("X,Y"),
);

thead.append(filaEncabezado);

const tbody = document.createElement("tbody");

tabla.append(thead, tbody);

// Bug 1: la tabla en sí no scrollea — este contenedor es el que
// tiene overflow-y y ocupa el espacio disponible de la card, dejando
// que la tabla crezca a su altura natural adentro (mismo patrón que
// vent_configuracion_main.ts::scrollTabla).
const scrollTabla = document.createElement("div");
scrollTabla.className = "coordenadas-tabla-scroll";
scrollTabla.append(tabla);

card.append(barraSuperior, scrollTabla);
raiz.append(card);
raiz.append(crearContenedorPopup());

// ======================================================
// 📌 FIJAR VENTANA
// ------------------------------------------------------
// Arranca siempre sin fijar — no se recuerda entre aperturas.
// ======================================================

let ventanaFijada = false;

botonFijar.addEventListener("click", () => {
  ventanaFijada = !ventanaFijada;
  botonFijar.classList.toggle("coordenadas-btn-fijar-activo", ventanaFijada);
});

// ======================================================
// ⁝⁝ ARRASTRAR Y SOLTAR (util_arrastrable.ts)
// ======================================================

function obtenerOrdenIds(): string[] {
  return Array.from(tbody.children).map(
    (fila) => (fila as HTMLElement).dataset.id ?? "",
  );
}

function onReordenar(nuevoOrden: string[]): void {
  void invoke("coordenadas_reordenar", { orden: nuevoOrden }).then(cargarLista);
}

const controladorArrastre = crearControladorArrastre({
  contenedor: tbody,
  obtenerOrdenIds,
  onReordenar,
});

// ======================================================
// 📋 CARGAR / RENDERIZAR LISTA
// ======================================================

async function cargarLista(): Promise<void> {
  const aplicacion = filtroGrupo || undefined;
  const tipo = filtroTipo ? Number(filtroTipo) : undefined;

  const filas = await invoke<CoordenadaBancoJson[]>("coordenadas_listar", {
    aplicacion,
    tipo,
    modo: undefined,
  });

  listaFiltradaActual = filas.map(convertirCoordenadaBanco);
  renderizarTabla(listaFiltradaActual);
}

async function cargarGruposFiltro(): Promise<void> {
  gruposDisponibles = await invoke<string[]>("coordenadas_listar_grupos");
}

// Última lista filtrada renderizada.
let listaFiltradaActual: CoordenadaBanco[] = [];

// Ids de coordenadas cuyo marcador aún no fue generado (Etapa E) —
// muestran "Generar marcador ⊙" en la columna X,Y en vez de los dos
// botones de coordenada.
const idsSinMarcador = new Set<string>();

// ======================================================
// ➕ AGREGAR FILA — Regla 7
// ======================================================

async function agregarFila(): Promise<void> {
  const nueva = crearCoordenadaBanco();

  if (filtroGrupo) {
    nueva.aplicacion = filtroGrupo;
  }

  if (filtroTipo) {
    nueva.tipo = Number(filtroTipo);
  }

  if (nueva.tipo === 3) {
    nueva.modo = 1;
    nueva.puntoReferencia = 1;
  }

  const guardada = await invoke<CoordenadaBancoJson>("coordenadas_agregar", {
    coordenada: coordenadaBancoParaBackend(nueva),
  });

  idsSinMarcador.add(guardada.id);

  await cargarLista();
}

// Ids de coordenadas con previsualización (marcador "⊙") activa —
// cada una en su propia ventana overlay. Etapa F: reemplaza el
// idPrevisualizado único (Etapa E) porque ahora puede haber
// cualquier cantidad activas a la vez.
const idsPrevisualizados = new Set<string>();

// Id de la coordenada con el círculo verde de selección activo — Etapa H.
let idSeleccionado: string | null = null;

// ======================================================
// 👁️ PREVISUALIZACIÓN — Etapa F
// ======================================================

async function cerrarPrevisualizacion(): Promise<void> {
  if (idsPrevisualizados.size === 0) {
    return;
  }

  const ids = Array.from(idsPrevisualizados);
  idsPrevisualizados.clear();

  detenerPollingXYEnVivo();

  await Promise.all(
    ids.map((id) =>
      invoke("cerrar_ventana_preview_coordenada", { id }).catch(() => {}),
    ),
  );
}

async function abrirPrevisualizacion(
  coordenada: CoordenadaBanco,
  numero: number,
): Promise<void> {
  idsPrevisualizados.add(coordenada.id);

  await invoke("abrir_ventana_preview_coordenada", {
    id: coordenada.id,
    numero,
    ubicacion: TIPO_A_UBICACION[coordenada.tipo] ?? "absoluta",
    modoVentana: MODO_A_MODO_VENTANA[coordenada.modo] ?? "pixeles",
    puntoReferencia:
      PUNTO_REFERENCIA_NUMERO_A_STRING[coordenada.puntoReferencia] ?? "sup_izq",
    x: coordenada.x,
    y: coordenada.y,
  });

  iniciarPollingXYEnVivo();
}

// ======================================================
// 🔄 ACTUALIZAR X,Y EN VIVO MIENTRAS SE ARRASTRA EL MARCADOR
// ------------------------------------------------------
// Bug 5: antes la columna X,Y solo se refrescaba al CERRAR la
// previsualización (cargarLista() en cerrarPrevisualizacionDe/
// alternarPrevisualizacion) — mientras el marcador seguía abierto,
// arrastrarlo no se reflejaba en la tabla hasta cerrarlo. Mismo
// patrón de polling que ya usa vent_captura_main.ts (Regla 17):
// mientras haya al menos una previsualización activa, cada tick
// consulta el x/y CRUDO en memoria (obtener_xy_preview_coordenada
// — ya actualizado por guardar_posicion_preview_coordenada tras
// cada arrastre, sin esperar a releer disco) y pisa solo el texto
// de los botones x/y de esa fila puntual — nunca re-renderiza toda
// la tabla (perdería edición en curso de otra fila, foco, etc.).
// ======================================================

let intervaloXYEnVivo: ReturnType<typeof setInterval> | null = null;

function iniciarPollingXYEnVivo(): void {
  if (intervaloXYEnVivo !== null) {
    return;
  }

  intervaloXYEnVivo = setInterval(() => void actualizarXYEnVivo(), 300);
}

function detenerPollingXYEnVivo(): void {
  if (intervaloXYEnVivo !== null) {
    clearInterval(intervaloXYEnVivo);
    intervaloXYEnVivo = null;
  }
}

async function actualizarXYEnVivo(): Promise<void> {
  await Promise.all(
    Array.from(idsPrevisualizados).map(async (id) => {
      let xy: [number, number] | null;

      try {
        xy = await invoke<[number, number] | null>(
          "obtener_xy_preview_coordenada",
          { id },
        );
      } catch {
        // Mismo criterio que actualizarPreview() en
        // vent_captura_main.ts: un error transitorio de IPC no debe
        // cortar el polling entero, solo se salta este tick para
        // esta fila.
        return;
      }

      if (!xy) {
        return;
      }

      const [x, y] = xy;

      const coordenada = listaFiltradaActual.find((c) => c.id === id);

      if (!coordenada) {
        return;
      }

      // Mantener el objeto en memoria sincronizado — si algo más
      // dispara renderizarTabla(listaFiltradaActual) mientras la
      // previsualización sigue abierta (ej. cambiar el filtro), la
      // celda recién creada debe partir del valor ya arrastrado, no
      // del viejo.
      coordenada.x = x;
      coordenada.y = y;

      const botonXY = document.querySelector<HTMLButtonElement>(
        `.coordenadas-boton-icono[data-id-coordenada="${id}"]`,
      );

      if (botonXY) {
        botonXY.textContent = textoXY(coordenada, x, y);
      }
    }),
  );
}

async function cerrarPrevisualizacionDe(id: string): Promise<void> {
  idsPrevisualizados.delete(id);

  if (idsPrevisualizados.size === 0) {
    detenerPollingXYEnVivo();
  }

  await invoke("cerrar_ventana_preview_coordenada", { id }).catch(() => {});
}

async function alternarPrevisualizacion(
  coordenada: CoordenadaBanco,
  numero: number,
): Promise<void> {
  if (idsPrevisualizados.has(coordenada.id)) {
    await cerrarPrevisualizacionDe(coordenada.id);
    // Regla 17: si el usuario arrastró el marcador antes de cerrar la
    // previsualización, x/y ya quedaron guardados en disco (Rust) —
    // se recarga la lista para reflejarlo en la columna X,Y.
    await cargarLista();
    return;
  }

  await abrirPrevisualizacion(coordenada, numero);

  // Regla 15: sin esto, el botón ⊙ de ESTA fila no queda marcado como
  // activo hasta que algún otro evento (cerrar una, o el botón global)
  // vuelva a renderizar la tabla completa.
  renderizarTabla(listaFiltradaActual);
}

// Regla 10: botón del encabezado ⊙️ — si hay alguna previsualización
// oculta (ninguna fila activa), muestra el marcador de todas las
// filas de la tabla filtrada actual; si ya están todas visibles, las
// oculta todas.
async function alternarPrevisualizacionGlobal(): Promise<void> {
  if (idsPrevisualizados.size > 0) {
    await cerrarPrevisualizacion();
    await cargarLista();
    return;
  }

  await Promise.all(
    listaFiltradaActual.map((coordenada, indice) =>
      abrirPrevisualizacion(coordenada, indice + 1),
    ),
  );

  await cargarLista();
}

// ======================================================
// ▶️ PROBAR — Etapa F
// ======================================================

function probarCoordenadaBanco(coordenada: CoordenadaBanco): void {
  invoke("probar_coordenada", {
    ubicacion: TIPO_A_UBICACION[coordenada.tipo] ?? "absoluta",
    modoVentana: MODO_A_MODO_VENTANA[coordenada.modo] ?? "pixeles",
    puntoReferencia:
      PUNTO_REFERENCIA_NUMERO_A_STRING[coordenada.puntoReferencia] ?? "sup_izq",
    x: coordenada.x,
    y: coordenada.y,
  }).catch(() => {});
}

function crearBotonProbar(coordenada: CoordenadaBanco): HTMLButtonElement {
  const boton = document.createElement("button");
  boton.type = "button";
  boton.className = "coordenadas-boton-icono coordenadas-icono-togglable";
  boton.textContent = "▶";
  boton.title = "Probar";

  if (coordenada.tipo !== 1) {
    boton.disabled = true;
    boton.classList.add("coordenadas-boton-icono-bloqueado");
    return boton;
  }

  boton.addEventListener("click", () => {
    probarCoordenadaBanco(coordenada);
  });
  return boton;
}

// ======================================================
// 📌 GENERAR MARCADOR — Etapa E
// ------------------------------------------------------
// Mismo mecanismo de captura ya usado en crearDetalleCoordenada
// (comp_popup_macro_editor.ts): abre la ventana overlay de
// captura y sondea obtener_resultado_coordenada cada 200ms.
// ======================================================

function iniciarCapturaMarcador(coordenada: CoordenadaBanco): void {
  invoke("abrir_ventana_captura_coordenada", {
    ubicacion: TIPO_A_UBICACION[coordenada.tipo] ?? "absoluta",
    modoVentana: MODO_A_MODO_VENTANA[coordenada.modo] ?? "pixeles",
    puntoReferencia:
      PUNTO_REFERENCIA_NUMERO_A_STRING[coordenada.puntoReferencia] ?? "sup_izq",
  }).catch((error) => {
    console.error("abrir_ventana_captura_coordenada FALLÓ:", error);
  });

  const intervalo = setInterval(() => {
    invoke<[number, number] | null>("obtener_resultado_coordenada")
      .then((resultado) => {
        if (!resultado) {
          return;
        }

        clearInterval(intervalo);

        coordenada.x = resultado[0];
        coordenada.y = resultado[1];
        idsSinMarcador.delete(coordenada.id);

        void invoke("coordenadas_editar", {
          id: coordenada.id,
          coordenada: coordenadaBancoParaBackend(coordenada),
        }).then(cargarLista);
      })
      .catch(() => {
        clearInterval(intervalo);
      });
  }, 200);
}

// Regla 14 (y Bug 5): en modo Porcentaje, máximo 2 decimales — usada
// tanto al renderizar la celda como al refrescarla en vivo durante
// el arrastre del marcador (ver actualizarXYEnVivo()).
function formatearEjeCoordenada(coordenada: CoordenadaBanco, valor: number): string {
  const esPorcentaje = coordenada.tipo === 3 && coordenada.modo === 2;
  return esPorcentaje ? valor.toFixed(2) : String(valor);
}

function crearCeldaXY(coordenada: CoordenadaBanco): HTMLTableCellElement {
  const td = document.createElement("td");
  td.className = "coordenadas-celda-xy";

  if (idsSinMarcador.has(coordenada.id)) {
    const botonGenerar = document.createElement("button");
    botonGenerar.type = "button";
    botonGenerar.className = "coordenadas-boton coordenadas-boton-primario";
    botonGenerar.textContent = "Generar marcador ⊙";
    botonGenerar.addEventListener("click", () => {
      iniciarCapturaMarcador(coordenada);
    });
    td.append(botonGenerar);

    return td;
  }

  const botonXY = document.createElement("button");
  botonXY.type = "button";
  botonXY.className = "coordenadas-boton-icono";
  botonXY.dataset.idCoordenada = coordenada.id;
  botonXY.textContent = textoXY(coordenada, coordenada.x, coordenada.y);
  botonXY.addEventListener("click", () => {
    iniciarCapturaMarcador(coordenada);
  });

  td.append(botonXY);

  return td;
}

// Punto 3: formato "(x,y)" en vez de "x: .. " / "y: .. " en dos
// botones separados.
function textoXY(coordenada: CoordenadaBanco, x: number, y: number): string {
  return `(${formatearEjeCoordenada(coordenada, x)},${formatearEjeCoordenada(coordenada, y)})`;
}

// ======================================================
// 🗔 POPUP TIPO — Absoluta / Cursor / Ventana
// ======================================================

const ICONO_PUNTO_REFERENCIA: Record<number, string> = {
  1: "⌜",
  2: "⌝",
  3: "⊡",
  4: "⌞",
  5: "⌟",
};

function textoBotonTipo(coordenada: CoordenadaBanco): string {
  if (coordenada.tipo !== 3) {
    return textoTipoCoordenada(coordenada.tipo);
  }

  if (coordenada.modo === 2) {
    return "Ventana: %";
  }

  return `Ventana: ${ICONO_PUNTO_REFERENCIA[coordenada.puntoReferencia] ?? "?"}`;
}

function abrirPopupTipo(
  evento: MouseEvent,
  coordenada: CoordenadaBanco,
  alCerrar: () => void,
): void {
  // Bug 4: solo el primer dibujado posiciona el popup (mostrarPopup,
  // que calcula el anclaje contra el punto de click). Los redibujados
  // siguientes (al elegir una opción) solo reemplazan el contenido en
  // el mismo lugar (actualizarContenidoPopup) — si se recalculara la
  // posición cada vez, un cambio de tamaño que cruce el borde de la
  // ventana hace "saltar" el popup a otro anclaje.
  let primerDibujado = true;

  const guardarCampo = (cambios: Partial<CoordenadaBanco>): void => {
    Object.assign(coordenada, cambios);

    void invoke("coordenadas_editar", {
      id: coordenada.id,
      coordenada: coordenadaBancoParaBackend(coordenada),
    });

    dibujar();
  };

  const dibujar = (): void => {
    const lista = document.createElement("div");
    lista.className = "popup-lista";

    lista.append(
      crearFilaPopup(
        "Tipo:",
        crearGrupoOpciones(
          [
            { texto: textoTipoCoordenada(1), valor: "1" },
            { texto: textoTipoCoordenada(2), valor: "2" },
            { texto: textoTipoCoordenada(3), valor: "3" },
          ],
          String(coordenada.tipo),
          (tipo) => guardarCampo({ tipo: Number(tipo) }),
        ),
      ),
    );

    if (coordenada.tipo === 3) {
      const cajaMedidoEn = document.createElement("div");
      cajaMedidoEn.className = "popup-caja-interna";

      cajaMedidoEn.append(
        crearFilaPopup(
          "Medido en:",
          crearGrupoOpciones(
            [
              { texto: textoModoCoordenada(2), valor: "2" },
              { texto: textoModoCoordenada(1), valor: "1" },
            ],
            String(coordenada.modo),
            (modo) => guardarCampo({ modo: Number(modo) }),
          ),
        ),
      );

      if (coordenada.modo === 1) {
        const onSeleccionarPunto = (puntoReferencia: string): void =>
          guardarCampo({ puntoReferencia: Number(puntoReferencia) });

        cajaMedidoEn.append(
          crearFilaPopup(
            "Medido desde:",
            crearGrupoOpciones(
              [1, 2, 3].map((punto) => ({
                texto: `${textoPuntoReferenciaCoordenada(punto)}: ${ICONO_PUNTO_REFERENCIA[punto]}`,
                valor: String(punto),
              })),
              String(coordenada.puntoReferencia),
              onSeleccionarPunto,
            ),
          ),
        );

        cajaMedidoEn.append(
          crearGrupoOpciones(
            [4, 5].map((punto) => ({
              texto: `${textoPuntoReferenciaCoordenada(punto)}: ${ICONO_PUNTO_REFERENCIA[punto]}`,
              valor: String(punto),
            })),
            String(coordenada.puntoReferencia),
            onSeleccionarPunto,
          ),
        );
      }

      lista.append(cajaMedidoEn);
    }

    if (primerDibujado) {
      primerDibujado = false;
      mostrarPopup(lista, evento.clientX, evento.clientY, alCerrar);
      return;
    }

    actualizarContenidoPopup(lista);
  };

  dibujar();
}

// ======================================================
// ✅ SELECCIÓN HACIA EL LLAMADOR — Etapa H
// ======================================================

// Regla 9: mensaje flotante junto al mouse al enviar la selección con
// la ventana fijada (si no está fijada, la ventana se cierra sola —
// no hace falta confirmación visual).
function mostrarToastEnviado(clientX: number, clientY: number): void {
  const toast = document.createElement("div");
  toast.className = "coordenadas-toast-enviado";
  toast.textContent = "✓ Enviado";
  toast.style.left = `${clientX + 12}px`;
  toast.style.top = `${clientY + 12}px`;

  raiz.append(toast);

  setTimeout(() => {
    toast.remove();
  }, 1000);
}

async function seleccionarFila(
  coordenada: CoordenadaBanco,
  evento: MouseEvent,
): Promise<void> {
  idSeleccionado = coordenada.id;

  await invoke("seleccionar_coordenada_banco", {
    coordenada: coordenadaBancoParaBackend(coordenada),
  });

  renderizarTabla(listaFiltradaActual);

  if (!ventanaFijada) {
    await getCurrentWindow().close();
    return;
  }

  mostrarToastEnviado(evento.clientX, evento.clientY);
}

function renderizarTabla(lista: CoordenadaBanco[]): void {
  tbody.innerHTML = "";

  lista.forEach((coordenada, indice) => {
    const tr = crearFila(coordenada, indice);
    tbody.append(tr);

    const asa = tr.querySelector<HTMLElement>(".coordenadas-asa");

    if (asa) {
      controladorArrastre.registrarFila(coordenada.id, tr, asa);
    }
  });
}

// Celda de texto editable: muestra un <span>; al hacer click lo
// reemplaza por un <input> con foco; al perder foco o presionar
// Enter, guarda y vuelve a mostrar el <span>.
function crearCeldaEditable(
  valorInicial: string,
  onGuardar: (valor: string) => void,
): HTMLTableCellElement {
  const td = document.createElement("td");

  const texto = document.createElement("span");
  texto.textContent = valorInicial;
  texto.className = "coordenadas-celda-texto";
  texto.classList.toggle("coordenadas-celda-texto--vacia", valorInicial === "");

  const entrarEnEdicion = (): void => {
    const input = document.createElement("input");
    input.type = "text";
    // Bug 3: sin esto, el ancho mínimo por defecto del <input> (basado
    // en el atributo "size", 20 por defecto) fuerza a la tabla a
    // ensanchar la columna al entrar en edición. Con size=1, el ancho
    // real lo sigue dando el CSS (width:100% de .coordenadas-celda-input).
    input.size = 1;
    input.value = texto.textContent ?? "";
    input.className = "coordenadas-celda-input";

    const confirmar = (): void => {
      const valor = input.value;
      texto.textContent = valor;
      texto.classList.toggle("coordenadas-celda-texto--vacia", valor === "");
      td.replaceChildren(texto);
      onGuardar(valor);
    };

    input.addEventListener("blur", confirmar);
    input.addEventListener("keydown", (evento) => {
      if (evento.key === "Enter") {
        input.blur();
      }
    });

    td.replaceChildren(input);
    input.focus();
  };

  texto.addEventListener("click", entrarEnEdicion);
  td.append(texto);

  return td;
}

// Regla 11: celda de la columna Grupo — mismo comportamiento de
// crearCeldaEditable (click → input, escribe nombre nuevo), más un
// popup compacto con los grupos ya existentes para elegir con un
// click sin tener que escribirlo.
function crearCeldaGrupo(
  valorInicial: string,
  onGuardar: (valor: string) => void,
): HTMLTableCellElement {
  const td = document.createElement("td");

  const texto = document.createElement("span");
  texto.textContent = valorInicial;
  texto.className = "coordenadas-celda-texto";
  texto.classList.toggle("coordenadas-celda-texto--vacia", valorInicial === "");

  const confirmarValor = (valor: string): void => {
    texto.textContent = valor;
    texto.classList.toggle("coordenadas-celda-texto--vacia", valor === "");
    td.replaceChildren(texto);
    onGuardar(valor);
  };

  const entrarEnEdicion = (evento: MouseEvent): void => {
    const input = document.createElement("input");
    input.type = "text";
    // Bug 3: ver comentario equivalente en crearCeldaEditable.
    input.size = 1;
    input.value = texto.textContent ?? "";
    input.className = "coordenadas-celda-input";

    input.addEventListener("blur", () => confirmarValor(input.value));
    input.addEventListener("keydown", (eventoTecla) => {
      // Bug 2: al empezar a escribir, el popup de grupos ya creados
      // (mostrado más abajo) debe desaparecer con el down de cualquier
      // tecla, no solo al confirmar con Enter.
      ocultarPopup();

      if (eventoTecla.key === "Enter") {
        input.blur();
      }
    });

    td.replaceChildren(input);
    input.focus();

    void cargarGruposFiltro().then(() => {
      if (gruposDisponibles.length === 0) {
        return;
      }

      const lista = document.createElement("div");
      lista.className = "popup-lista coordenadas-popup-grupo-compacto";

      lista.append(
        crearGrupoOpciones(
          gruposDisponibles.map((grupo) => ({ texto: grupo, valor: grupo })),
          texto.textContent ?? "",
          (valor) => {
            confirmarValor(valor);
            ocultarPopup();
          },
          "popup-grupo-vertical",
        ),
      );

      mostrarPopup(lista, evento.clientX, evento.clientY);
    });
  };

  texto.addEventListener("click", entrarEnEdicion);
  td.append(texto);

  return td;
}

function crearFila(
  coordenada: CoordenadaBanco,
  indice: number,
): HTMLTableRowElement {
  const tr = document.createElement("tr");
  tr.dataset.id = coordenada.id;

  const tdNumero = document.createElement("td");
  tdNumero.className = "coordenadas-numero-celda";
  const botonNumero = document.createElement("button");
  botonNumero.type = "button";
  botonNumero.className = "coordenadas-numero-toggle";
  botonNumero.title = "Seleccionar esta fila";
  botonNumero.classList.toggle(
    "coordenadas-numero-toggle-activo",
    coordenada.id === idSeleccionado,
  );
  const spanNumero = document.createElement("span");
  spanNumero.textContent = String(indice + 1);
  botonNumero.append(spanNumero);
  botonNumero.addEventListener("click", (evento) => {
    void seleccionarFila(coordenada, evento);
  });
  tdNumero.append(botonNumero);
  tr.append(tdNumero);

  // Bug 5: el flex va en un div INTERNO, no en el <td> mismo — igual
  // que el encabezado (celdaOpcionesEncabezado). Poner display:flex
  // directo sobre el <td> le hace perder su layout de celda de tabla,
  // y la columna termina calculando un ancho distinto al del
  // encabezado (el borde derecho queda corrido, a mitad de los botones).
  const tdOpciones = document.createElement("td");

  const cajaOpciones = document.createElement("div");
  cajaOpciones.className = "coordenadas-celda-opciones";

  const botonAsa = document.createElement("button");
  botonAsa.type = "button";
  botonAsa.className = "coordenadas-asa";
  botonAsa.textContent = "⁝";
  botonAsa.addEventListener("click", () => {
    tabla.classList.toggle("coordenadas-tabla--iconos-desplegados");
  });

  const botonPreview = document.createElement("button");
  botonPreview.type = "button";
  botonPreview.className =
    "coordenadas-boton-icono coordenadas-icono-togglable";
  botonPreview.textContent = "⊙";
  botonPreview.title = "Previsualizar";
  botonPreview.classList.toggle(
    "coordenadas-boton-icono-activo",
    idsPrevisualizados.has(coordenada.id),
  );
  botonPreview.addEventListener("click", () => {
    void alternarPrevisualizacion(coordenada, indice + 1);
  });

  const botonEliminar = document.createElement("button");
  botonEliminar.type = "button";
  botonEliminar.className =
    "coordenadas-boton-icono coordenadas-icono-togglable coordenadas-boton-eliminar";
  botonEliminar.textContent = "x";
  botonEliminar.addEventListener("click", async () => {
    await invoke("coordenadas_eliminar", { id: coordenada.id });
    await cargarLista();
  });

  cajaOpciones.append(
    botonAsa,
    botonPreview,
    crearBotonProbar(coordenada),
    botonEliminar,
  );
  tdOpciones.append(cajaOpciones);
  tr.append(tdOpciones);

  tr.append(
    crearCeldaGrupo(coordenada.aplicacion, (valor) => {
      void invoke("coordenadas_editar", {
        id: coordenada.id,
        coordenada: coordenadaBancoParaBackend({
          ...coordenada,
          aplicacion: valor,
        }),
      }).then(cargarLista);
    }),
  );

  tr.append(
    crearCeldaEditable(coordenada.nota, (valor) => {
      void invoke("coordenadas_editar", {
        id: coordenada.id,
        coordenada: coordenadaBancoParaBackend({
          ...coordenada,
          nota: valor,
        }),
      }).then(cargarLista);
    }),
  );

  const tdTipo = document.createElement("td");
  const botonTipo = document.createElement("button");
  botonTipo.type = "button";
  botonTipo.className = "coordenadas-boton-icono";
  botonTipo.textContent = textoBotonTipo(coordenada);
  botonTipo.addEventListener("click", (evento) => {
    abrirPopupTipo(evento, coordenada, () => {
      void cargarLista();
    });
  });
  tdTipo.append(botonTipo);
  tr.append(tdTipo);

  tr.append(crearCeldaXY(coordenada));

  return tr;
}

// ======================================================
// 🔎 FILTROS EN VIVO
// ======================================================

function recargarPorFiltro(): void {
  void cerrarPrevisualizacion().then(cargarLista);
}

// Al cerrar la ventana, no dejar el marcador huérfano en pantalla.
getCurrentWindow().onCloseRequested(() => {
  void cerrarPrevisualizacion();
});

// ======================================================
// 🏁 INICIAR
// ======================================================

void cargarGruposFiltro();
cargarLista();
