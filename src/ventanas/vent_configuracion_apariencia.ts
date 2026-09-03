// ======================================================
// 🎨 vent_configuracion_Apariencia
// ------------------------------------------------------
// Tabla en árbol de la pestaña Apariencia (Nombre / Valor
// por Defecto / Editar / Valor Personalizado), separada de
// crearPestanaEditable() porque el catálogo ya no es una
// lista plana de filas sino un árbol Título/Subtítulo/
// Elemento con valores anidados (ver configuracion_listar_apariencia
// en comandos.rs).
//
// Placeholder de esqueleto: monta el árbol en la tabla nueva,
// pero hayEdicionesPendientes/validarYRecolectar/aplicarGuardado/
// marcarErroresGuardado/restablecerPestana todavía no tienen
// lógica real (llega en una etapa posterior de persistencia).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import type { Columna } from "../ui/ui_columnas";

import { activarRedimensionColumnas } from "../ui/ui_redimension_columnas";

import {
  mostrarPopup,
  ocultarPopup,
} from "../componentes/comp_popup_contenedor";

import { abrirFormularioNombre } from "../componentes/comp_popup_formulario_nombre";

import {
  crearFilaPopup,
  crearGrupoOpciones,
} from "../componentes/comp_popup_grupo";

import type {
  CambioConfiguracion,
  ErrorConfiguracion,
  Pestana,
  ResultadoGuardado,
} from "./vent_configuracion_main";

// Modelo tal cual lo entrega configuracion_listar_apariencia
// (snake_case) — tipo/valor_defecto son null en filas de nivel 1/2/3.
interface FilaCssCruda {
  id: string;
  nivel: number;
  nombre_ui: string;
  tipo: string | null;
  valor_defecto: string | null;
  valor_personalizado: string | null;
}

// Modelo tal cual lo entrega configuracion_tema_listar (Etapa F).
interface TemaListadoUI {
  nombre: string;
  origen: string;
}

// Fila de nivel 1/2/3 con sus valores de nivel 0 (hijos) ya agrupados.
interface NodoArbol {
  entrada: FilaCssCruda;
  hijos: FilaCssCruda[];
}

// Sentinel de "valor" para CambioConfiguracion cuando lo que hay que
// hacer es BORRAR el override de esa clave (no guardar un valor
// nuevo) — ver validarYRecolectar/aplicarGuardado más abajo. No se
// puede ampliar CambioConfiguracion con un variante propia (tipo
// compartido con General/Teclas en vent_configuracion_main.ts, que
// solo reenvía la lista sin inspeccionarla), así que se codifica acá
// con un valor que ningún tipo real (color/pixeles/texto/porcentaje/
// modo) puede producir.
const SENTINEL_BORRAR = "__borrar__";

// Fila ya montada en el DOM, guardada en orden (Etapa E8) para que el
// botón Expandir/Contraer de una fila de nivel 1 pueda ocultar/mostrar
// el tramo de filas que le siguen hasta el próximo nivel 1 (o el final
// de la tabla) sin recorrer el árbol de nuevo.
interface FilaMontada {
  nodo: NodoArbol;
  tr: HTMLTableRowElement;
  actualizarColumnas: () => void;
}

// ----------------------------------------------------
// Columnas de la tabla en árbol (Etapa E3/E4)
// ----------------------------------------------------

const COLUMNAS_ARBOL: Columna[] = [
  {
    id: "nombre",
    titulo: "Nombre",
    grupo: "general",
    ancho: "var(--col-config-nombre)",
  },

  {
    id: "defecto",
    titulo: "Valor por Defecto",
    grupo: "general",
    ancho: "var(--col-config-defecto)",
  },

  {
    id: "personalizado",
    titulo: "Valor Personalizado",
    grupo: "general",
    ancho: "var(--col-config-personalizado)",
  },
];

const ANCHOS_DEFAULT_ARBOL: Record<string, number> = {
  nombre: 220,
  defecto: 260,
  personalizado: 260,
};

// Sangría por nivel del árbol (Etapa E6): mayor jerarquía = menos
// sangría (Regla 32).
const SANGRIA_POR_NIVEL: Record<number, string> = {
  1: "0",
  2: "20px",
  3: "40px",
};

// ----------------------------------------------------
// Mapa de colores del tema (Etapa F1)
// ----------------------------------------------------
// Recorre el arreglo plano en busca de la sección "Color de tema"
// (nivel 1) y junta, de cada nodo dentro de ese tramo, sus hijos de
// tipo "color" en un mapa hex -> nombre_ui del nodo contenedor —
// usado para mostrar el NOMBRE del color en vez de su hex (Regla 33).

function construirMapaColoresTema(arbol: NodoArbol[]): Map<string, string> {
  const mapa = new Map<string, string>();

  const indiceInicio = arbol.findIndex(
    (nodo) => nodo.entrada.id === "color-tema" && nodo.entrada.nivel === 1,
  );

  if (indiceInicio === -1) {
    return mapa;
  }

  for (let i = indiceInicio + 1; i < arbol.length; i++) {
    const nodo = arbol[i];

    if (nodo.entrada.nivel === 1) {
      break;
    }

    for (const hijo of nodo.hijos) {
      if (hijo.tipo === "color" && hijo.valor_defecto) {
        mapa.set(hijo.valor_defecto.toLowerCase(), nodo.entrada.nombre_ui);
      }
    }
  }

  return mapa;
}

// ----------------------------------------------------
// Construcción del árbol
// ----------------------------------------------------

function construirArbol(filas: FilaCssCruda[]): NodoArbol[] {
  const nodos: NodoArbol[] = [];

  let actual: NodoArbol | null = null;

  for (const fila of filas) {
    if (fila.nivel === 0) {
      actual?.hijos.push(fila);
      continue;
    }

    actual = { entrada: fila, hijos: [] };
    nodos.push(actual);
  }

  return nodos;
}

// ----------------------------------------------------
// Filtro de árbol por Título de nivel 1 (Apariencia vs Tema: cada
// pestaña muestra solo un subconjunto de los Títulos del catálogo,
// ver crearPestanaApariencia). Conserva el tramo completo (nivel
// 2/3 + sus hijos nivel 0) de cada Título permitido.
// ----------------------------------------------------

function filtrarArbolPorGrupos(
  arbol: NodoArbol[],
  gruposPermitidos: Set<string>,
): NodoArbol[] {
  const resultado: NodoArbol[] = [];

  let manteniendo = false;

  for (const nodo of arbol) {
    if (nodo.entrada.nivel === 1) {
      manteniendo = gruposPermitidos.has(nodo.entrada.id);
    }

    if (manteniendo) {
      resultado.push(nodo);
    }
  }

  return resultado;
}

// ----------------------------------------------------
// Preset de Escala general (Pequeño/Normal/Grande) — pestaña
// Apariencia. Fija de una sola vez el tamaño de texto (fs11/fs13/
// fs16) y las dimensiones (toolbar-height/row-height/r6/gap8/gap16);
// deja el resultado como edición pendiente igual que un campo tocado
// a mano (no persiste solo).
// ----------------------------------------------------

type ClaveEscala = "pequeno" | "normal" | "grande";

const PRESETS_ESCALA: Record<ClaveEscala, Record<string, string>> = {
  pequeno: {
    fs11: "10px",
    fs13: "12px",
    fs16: "14px",
    "toolbar-height": "42px",
    "row-height": "34px",
    r6: "4px",
    gap8: "6px",
    gap16: "12px",
  },
  normal: {
    fs11: "11px",
    fs13: "13px",
    fs16: "16px",
    "toolbar-height": "48px",
    "row-height": "40px",
    r6: "6px",
    gap8: "8px",
    gap16: "16px",
  },
  grande: {
    fs11: "13px",
    fs13: "15px",
    fs16: "19px",
    "toolbar-height": "54px",
    "row-height": "46px",
    r6: "8px",
    gap8: "10px",
    gap16: "20px",
  },
};

// ----------------------------------------------------
// Expandir/Contraer (Etapa E8)
// ----------------------------------------------------

function alternarExpandir(
  boton: HTMLButtonElement,
  tr: HTMLTableRowElement,
  filasMontadas: FilaMontada[],
): void {
  const indiceActual = filasMontadas.findIndex((f) => f.tr === tr);

  if (indiceActual === -1) {
    return;
  }

  let indiceFin = filasMontadas.length;

  for (let i = indiceActual + 1; i < filasMontadas.length; i++) {
    if (filasMontadas[i].nodo.entrada.nivel === 1) {
      indiceFin = i;
      break;
    }
  }

  const contraer = boton.textContent === "▾";

  boton.textContent = contraer ? "▸" : "▾";

  for (let i = indiceActual + 1; i < indiceFin; i++) {
    filasMontadas[i].tr.classList.toggle("oculta", contraer);
  }
}

// ----------------------------------------------------
// Render de Valor por Defecto (Etapa F3)
// ----------------------------------------------------
// Swatch + nombre de tema para colores; texto plano para el resto
// de tipos (porcentaje/pixeles/texto/modo). Hijos separados por " ; ".

function renderizarValorDefecto(
  hijos: FilaCssCruda[],
  coloresTema: Map<string, string>,
): DocumentFragment {
  const fragmento = document.createDocumentFragment();

  const conValor = hijos.filter(
    (hijo) => hijo.valor_defecto && hijo.valor_defecto.length > 0,
  );

  conValor.forEach((hijo, indice) => {
    const valor = hijo.valor_defecto as string;

    if (hijo.tipo === "color") {
      const swatch = document.createElement("span");
      swatch.className = "configuracion-arbol-swatch";
      swatch.style.backgroundColor = valor;

      const nombre = document.createElement("span");
      nombre.textContent = coloresTema.get(valor.toLowerCase()) ?? valor;

      fragmento.append(swatch, nombre);
    } else {
      fragmento.append(document.createTextNode(valor));
    }

    if (indice < conValor.length - 1) {
      fragmento.append(document.createTextNode(" ; "));
    }
  });

  return fragmento;
}

// ----------------------------------------------------
// Campo de edición según tipo (Etapa G2)
// ----------------------------------------------------

function crearCampoValor(
  hijo: FilaCssCruda,
  alCambiar: (valorFormateado: string) => void,
): HTMLElement {
  const actual = hijo.valor_personalizado ?? hijo.valor_defecto ?? "";

  if (hijo.tipo === "color") {
    const input = document.createElement("input");
    input.type = "color";
    input.value = actual;
    input.addEventListener("input", () => alCambiar(input.value));
    return input;
  }

  if (hijo.tipo === "porcentaje") {
    const input = document.createElement("input");
    input.type = "number";
    input.min = "0";
    input.max = "100";
    input.step = "1";
    input.value = actual.replace("%", "");
    input.addEventListener("input", () => alCambiar(`${input.value}%`));
    return input;
  }

  if (hijo.tipo === "pixeles") {
    const input = document.createElement("input");
    input.type = "number";
    input.min = "0";
    input.step = "1";
    input.value = actual.replace("px", "");
    input.addEventListener("input", () => alCambiar(`${input.value}px`));
    return input;
  }

  if (hijo.tipo === "modo") {
    const opciones: { texto: string; valor: string }[] = [
      { texto: "Plano", valor: "plano" },
      { texto: "Degradado", valor: "degradado" },
    ];

    return crearGrupoOpciones(opciones, actual, (valor) => alCambiar(valor));
  }

  const input = document.createElement("input");
  input.type = "text";
  input.value = actual;
  input.addEventListener("input", () => alCambiar(input.value));
  return input;
}

// ----------------------------------------------------
// Popup Editar (Etapa G3)
// ----------------------------------------------------

function crearPopupEditar(
  nodo: NodoArbol,
  actualizarColumnas: () => void,
): HTMLElement {
  const popup = document.createElement("div");
  popup.className = "popup-editar-apariencia";

  nodo.hijos.forEach((hijo) => {
    const campo = crearCampoValor(hijo, (valorFormateado) => {
      hijo.valor_personalizado = valorFormateado;
      actualizarColumnas();
    });

    popup.append(crearFilaPopup(hijo.nombre_ui, campo));
  });

  return popup;
}

// ----------------------------------------------------
// Render de Valor Personalizado (Etapa G4)
// ----------------------------------------------------
// Misma lógica que renderizarValorDefecto (swatch+nombre de tema
// para color, texto plano para el resto) pero a partir de
// valor_personalizado y sin etiquetas (Regla 37).

function renderizarValorPersonalizado(
  hijos: FilaCssCruda[],
  coloresTema: Map<string, string>,
): DocumentFragment {
  const fragmento = document.createDocumentFragment();

  const conValor = hijos.filter(
    (hijo) => hijo.valor_personalizado && hijo.valor_personalizado.length > 0,
  );

  conValor.forEach((hijo, indice) => {
    const valor = hijo.valor_personalizado as string;

    if (hijo.tipo === "color") {
      const swatch = document.createElement("span");
      swatch.className = "configuracion-arbol-swatch";
      swatch.style.backgroundColor = valor;

      const nombre = document.createElement("span");
      nombre.textContent = coloresTema.get(valor.toLowerCase()) ?? valor;

      fragmento.append(swatch, nombre);
    } else {
      fragmento.append(document.createTextNode(valor));
    }

    if (indice < conValor.length - 1) {
      fragmento.append(document.createTextNode(" ; "));
    }
  });

  return fragmento;
}

// ----------------------------------------------------
// Montar una fila del árbol
// ----------------------------------------------------

function crearFilaArbol(
  nodo: NodoArbol,
  filasMontadas: FilaMontada[],
  coloresTema: Map<string, string>,
  filasConCambio: Set<NodoArbol>,
  valoresOriginales: Map<string, string | null>,
): { tr: HTMLTableRowElement; actualizarColumnas: () => void } {
  const tr = document.createElement("tr");

  const nivel = nodo.entrada.nivel;

  // A3/2-3: Título (nivel 1) no tiene valores propios (hijos siempre
  // vacíos, ver construirArbol) — se arma como una sola celda que
  // ocupa las 3 columnas (mismo patrón que montarFilaSubtitulo en
  // Teclas), sin división de columnas: así el nombre tiene todo el
  // ancho de la fila disponible antes de partirse en dos líneas.
  if (nivel === 1) {
    tr.classList.add("configuracion-arbol-titulo");

    const tdTitulo = document.createElement("td");
    tdTitulo.className = "configuracion-arbol-nivel-1";
    tdTitulo.colSpan = COLUMNAS_ARBOL.length;

    const botonExpandir = document.createElement("button");
    botonExpandir.type = "button";
    botonExpandir.className = "configuracion-arbol-expandir";
    botonExpandir.textContent = "▾";

    botonExpandir.addEventListener("click", () => {
      alternarExpandir(botonExpandir, tr, filasMontadas);
    });

    tdTitulo.append(
      botonExpandir,
      document.createTextNode(nodo.entrada.nombre_ui),
    );

    tr.append(tdTitulo);

    return { tr, actualizarColumnas: () => {} };
  }

  const tdNombre = document.createElement("td");
  tdNombre.classList.add(`configuracion-arbol-nivel-${nivel}`);
  tdNombre.style.paddingLeft = SANGRIA_POR_NIVEL[nivel] ?? "0";
  tdNombre.textContent = nodo.entrada.nombre_ui;

  // F4: swatches+nombre de tema en vez del join de texto plano
  // (poblado por actualizarColumnas() más abajo, junto con
  // Valor Personalizado).
  const tdDefecto = document.createElement("td");
  tdDefecto.className = "configuracion-arbol-defecto";

  // Columna Editar/Valor Personalizado fusionada (Etapa H11): un
  // solo botón por fila, sin padding propio en la celda (el botón
  // ocupa todo el ancho, ver .configuracion-arbol-personalizado-celda).
  const tdPersonalizado = document.createElement("td");
  tdPersonalizado.className = "configuracion-arbol-personalizado-celda";

  const botonPersonalizado = document.createElement("button");
  botonPersonalizado.type = "button";
  botonPersonalizado.className = "configuracion-arbol-personalizado";
  tdPersonalizado.append(botonPersonalizado);

  // G5/H11: refresca Valor por Defecto y el botón fusionado tras un
  // cambio en el popup Editar o un borrado por doble click — el
  // botón muestra "✎" vacío cuando no hay Valor Personalizado, o el
  // valor ya guardado (swatch+nombre para color) en su lugar.
  const actualizarColumnas = (): void => {
    tdDefecto.replaceChildren(renderizarValorDefecto(nodo.hijos, coloresTema));

    const hayValorPersonalizado = nodo.hijos.some(
      (hijo) => hijo.valor_personalizado && hijo.valor_personalizado.length > 0,
    );

    if (hayValorPersonalizado) {
      botonPersonalizado.replaceChildren(
        renderizarValorPersonalizado(nodo.hijos, coloresTema),
      );
    } else {
      botonPersonalizado.textContent = "✎";
    }
  };

  // F5/H1: doble click sobre Valor por Defecto borra el Valor
  // Personalizado de toda la fila (Regla 34). Si alguno de los hijos
  // tenía un valor original (ya guardado en el backend), la fila
  // sigue marcada en filasConCambio para que "Guardar cambios" mande
  // el borrado — ver SENTINEL_BORRAR en validarYRecolectar. Si
  // ninguno tenía valor original (todo era edición sin guardar
  // todavía), no hay nada que persistir: se saca del set.
  tdDefecto.addEventListener("dblclick", () => {
    nodo.hijos.forEach((hijo) => {
      hijo.valor_personalizado = null;
    });

    const habiaValorGuardado = nodo.hijos.some(
      (hijo) => valoresOriginales.get(hijo.id) !== null,
    );

    if (habiaValorGuardado) {
      filasConCambio.add(nodo);
      tr.classList.add("configuracion-arbol-editando");
    } else {
      filasConCambio.delete(nodo);
      tr.classList.remove("configuracion-arbol-editando");
    }

    actualizarColumnas();
  });

  // G6/H1/H11: botón fusionado (vacío=lápiz o con el Valor
  // Personalizado ya guardado) abre el mini popup sobre la fila;
  // cualquier cambio dentro del popup marca la fila como pendiente
  // de guardar. Se reabre igual estando vacío o con valor.
  botonPersonalizado.addEventListener("click", (evento) => {
    const popup = crearPopupEditar(nodo, () => {
      filasConCambio.add(nodo);
      tr.classList.add("configuracion-arbol-editando");

      actualizarColumnas();
    });

    mostrarPopup(popup, evento.clientX, evento.clientY);
  });

  actualizarColumnas();

  tr.append(tdNombre, tdDefecto, tdPersonalizado);

  return { tr, actualizarColumnas };
}

// ----------------------------------------------------
// Armar la tabla (thead de 4 columnas + tbody + scroll)
// ----------------------------------------------------

function crearTablaApariencia(panel: HTMLDivElement): HTMLTableSectionElement {
  const tabla = document.createElement("table");
  tabla.className = "configuracion-tabla-arbol";

  const thead = document.createElement("thead");
  const trEncabezado = document.createElement("tr");

  for (const col of COLUMNAS_ARBOL) {
    const th = document.createElement("th");
    th.className = "configuracion-arbol-celda";
    th.dataset.columna = col.id;
    th.style.width = col.ancho;
    th.textContent = col.titulo;
    trEncabezado.append(th);
  }

  thead.append(trEncabezado);

  // E4: reusa el mecanismo de arrastre de la tabla principal — la
  // tabla en árbol usa ".configuracion-arbol-celda" (no
  // ".cabecera-celda") como selector de celda, ver E1.
  activarRedimensionColumnas(
    trEncabezado,
    COLUMNAS_ARBOL,
    ANCHOS_DEFAULT_ARBOL,
    ".configuracion-arbol-celda",
  );

  const tbody = document.createElement("tbody");

  tabla.append(thead, tbody);

  const scrollTabla = document.createElement("div");
  scrollTabla.className = "configuracion-tabla-scroll";
  scrollTabla.append(tabla);

  panel.append(scrollTabla);

  return tbody;
}

// ----------------------------------------------------
// Pestaña completa
// ----------------------------------------------------

export interface OpcionesPestanaApariencia {
  // Ids de Título (nivel 1) del catálogo que muestra esta pestaña
  // (ver apariencia.tsv) — el resto del catálogo no se renderiza acá.
  grupos: string[];

  // Solo la pestaña "Tema" muestra el selector Cargar/Guardar/
  // Renombrar/Eliminar tema (botonSelectorTema, montado en la barra
  // global por vent_configuracion_main.ts vía elementoBarra).
  incluirSelectorTema: boolean;

  // Solo la pestaña "Apariencia" (Texto+Dimensiones) muestra el
  // selector de Escala general (Pequeño/Normal/Grande).
  incluirEscala: boolean;

  textoConfirmacionRestablecer: string;
}

export function crearPestanaApariencia(
  panel: HTMLDivElement,
  despuesDeAplicar: () => Promise<void>,
  opciones: OpcionesPestanaApariencia,
): Pestana & { refrescarDesdeOtraPestana: () => Promise<void> } {
  const gruposPermitidos = new Set(opciones.grupos);

  const tbody = crearTablaApariencia(panel);

  // E3: estado de sesión del selector de temas — nombre/origen del
  // tema que da la base de "Valor por Defecto" (ver
  // configuracion_apariencia_iniciar_sesion / configuracion_tema_cargar
  // en comandos.rs). Se completan en cargar().
  let nombreTemaSesion = "";
  let origenTemaSesion = "";

  // G1: si el tema de sesión tiene overrides css. vigentes — controla
  // la visibilidad de "Guardar valores editados" en el popup.
  let hayPersonalizadosSesion = false;

  // Se pone en true al elegir un tema en "Cargar ▾". A diferencia de
  // editar un valor puntual (filasConCambio), cargar un tema no marca
  // ninguna fila individual — sin este flag, hayEdicionesPendientes()
  // devolvía false y "Aplicar cambios" no detectaba nada que guardar.
  // Se apaga al reiniciar la sesión (cargar()) o tras guardar con
  // éxito (limpiarEstadoTrasGuardado()).
  let huboCargaDeTema = false;

  const botonSelectorTema = document.createElement("button");
  botonSelectorTema.type = "button";
  botonSelectorTema.className = "configuracion-selector-tema oculto";
  botonSelectorTema.title = "Selector de temas";

  botonSelectorTema.addEventListener("click", (evento) => {
    abrirPopupSelectorTema(evento);
  });

  // E4: refleja en el botón el tema de sesión + si hay overrides
  // pendientes/aplicados (nivel 0 con valor_personalizado no nulo).
  function actualizarBotonSelectorTema(filas: FilaCssCruda[]): void {
    const hayPersonalizados = filas.some(
      (f) => f.nivel === 0 && f.valor_personalizado !== null,
    );

    hayPersonalizadosSesion = hayPersonalizados;

    botonSelectorTema.textContent = hayPersonalizados
      ? `${nombreTemaSesion} (editado)`
      : nombreTemaSesion;
  }

  // F4: item de la lista de "Cargar ▾" — carga el tema en la sesión
  // (preview) y refresca la tabla sin reiniciar la sesión.
  function crearItemTema(
    tema: TemaListadoUI,
    alSeleccionar: () => void,
  ): HTMLButtonElement {
    const boton = document.createElement("button");

    boton.className = "ui-btn";
    boton.textContent = tema.nombre;

    boton.addEventListener("click", async () => {
      await invoke("configuracion_tema_cargar", {
        nombre: tema.nombre,
        origen: tema.origen,
      });

      nombreTemaSesion = tema.nombre;
      origenTemaSesion = tema.origen;
      huboCargaDeTema = true;

      await recargarTablaApariencia();

      alSeleccionar();
    });

    return boton;
  }

  // F5: caja interna de "Cargar ▾" — predefinidos, separador, temas
  // de usuario.
  async function crearCajaCargar(contenedor: HTMLElement): Promise<void> {
    const temas = await invoke<TemaListadoUI[]>("configuracion_tema_listar");

    const predefinidos = temas.filter((tema) => tema.origen === "predefinido");
    const usuario = temas.filter((tema) => tema.origen === "usuario");

    for (const tema of predefinidos) {
      contenedor.append(crearItemTema(tema, () => ocultarPopup()));
    }

    const separador = document.createElement("div");
    separador.className = "app-popup-separador";
    contenedor.append(separador);

    for (const tema of usuario) {
      contenedor.append(crearItemTema(tema, () => ocultarPopup()));
    }
  }

  // F6/G2: popup principal del selector de temas — "Cargar ▾"
  // (expandible en el mismo popup). "Guardar como" y "Renombrar"
  // (H4/H5) abren el popup reutilizado abrirFormularioNombre en vez
  // de expandirse en este mismo popup.
  function abrirPopupSelectorTema(evento: MouseEvent): void {
    let cargarExpandido = false;
    let confirmandoEliminar = false;

    const dibujar = (): void => {
      const lista = document.createElement("div");
      lista.className = "popup-lista";

      // ----------------------------------
      // Cargar ▾
      // ----------------------------------

      const botonCargar = document.createElement("button");
      botonCargar.className = "ui-btn";
      botonCargar.textContent = cargarExpandido ? "Cargar ▴" : "Cargar ▾";

      botonCargar.addEventListener("click", () => {
        cargarExpandido = !cargarExpandido;
        dibujar();
      });

      lista.append(botonCargar);

      if (cargarExpandido) {
        const caja = document.createElement("div");
        caja.className = "popup-caja-interna";

        lista.append(caja);

        crearCajaCargar(caja);
      }

      const separador = document.createElement("div");
      separador.className = "app-popup-separador";
      lista.append(separador);

      // ----------------------------------
      // Guardar como
      // ----------------------------------

      const botonGuardarComo = document.createElement("button");
      botonGuardarComo.className = "ui-btn";
      botonGuardarComo.textContent = "Guardar como";

      botonGuardarComo.addEventListener("click", () => {
        abrirFormularioNombre("", evento, async (nombre) => {
          await invoke("configuracion_tema_guardar_como", { nombre });
        });
      });

      lista.append(botonGuardarComo);

      // ----------------------------------
      // G2: opciones extra solo para tema de usuario
      // ----------------------------------

      if (origenTemaSesion === "usuario") {
        const separadorUsuario = document.createElement("div");
        separadorUsuario.className = "app-popup-separador";
        lista.append(separadorUsuario);

        // G3: Guardar valores editados
        if (hayPersonalizadosSesion) {
          const botonGuardarEditado = document.createElement("button");
          botonGuardarEditado.className = "ui-btn";
          botonGuardarEditado.textContent = "Guardar valores editados";

          botonGuardarEditado.addEventListener("click", async () => {
            await invoke("configuracion_tema_guardar_editado", {
              nombre: nombreTemaSesion,
            });

            await recargarTablaApariencia();

            ocultarPopup();
          });

          lista.append(botonGuardarEditado);
        }

        // G4/H5: Renombrar — reutiliza abrirFormularioNombre
        const botonRenombrar = document.createElement("button");
        botonRenombrar.className = "ui-btn";
        botonRenombrar.textContent = "Renombrar";

        botonRenombrar.addEventListener("click", () => {
          abrirFormularioNombre(
            nombreTemaSesion,
            evento,
            async (nombreNuevo) => {
              await invoke("configuracion_tema_renombrar", {
                nombreActual: nombreTemaSesion,
                nombreNuevo,
              });

              nombreTemaSesion = nombreNuevo;

              await recargarTablaApariencia();
            },
          );
        });

        lista.append(botonRenombrar);

        // G5: Eliminar Tema (doble verificación, mismo comportamiento
        // y color que "Eliminar perfil" de la barra lateral)
        const botonEliminarTema = document.createElement("button");
        botonEliminarTema.className = "ui-btn popup-perfil-eliminar";
        botonEliminarTema.textContent = confirmandoEliminar
          ? "⚠️ Confirmar eliminación"
          : "Eliminar Tema";

        botonEliminarTema.addEventListener("click", async () => {
          if (!confirmandoEliminar) {
            confirmandoEliminar = true;
            dibujar();

            return;
          }

          await invoke("configuracion_tema_eliminar", {
            nombre: nombreTemaSesion,
          });

          await cargar();

          ocultarPopup();
        });

        lista.append(botonEliminarTema);
      }

      mostrarPopup(lista, evento.clientX, evento.clientY);
    };

    dibujar();
  }

  // H2/H8: estado de edición pendiente, vive fuera de cargar() para
  // sobrevivir entre renders del árbol dentro de la misma sesión de
  // la ventana (se limpia explícitamente al recargar, ver cargar()).
  const filasConCambio = new Set<NodoArbol>();
  const trPorNodo = new Map<NodoArbol, HTMLTableRowElement>();

  // Valor personalizado tal cual llegó del backend al cargar (antes
  // de cualquier edición en memoria) — permite distinguir "no había
  // nada que borrar" de "había un override guardado que hay que
  // borrar" cuando el usuario hace doble click en Valor por Defecto.
  const valoresOriginales = new Map<string, string | null>();

  // Filas montadas de la última recarga — el preset de Escala
  // necesita ubicar los nodos por id (fs11/fs13/.../gap16) y
  // refrescar sus columnas tras aplicar un preset.
  let filasMontadasActual: FilaMontada[] = [];

  // Contenedor del grupo de botones Pequeño/Normal/Grande — se
  // completa en crearFilaEscala() (solo si opciones.incluirEscala),
  // se refresca en cada recarga vía actualizarSeleccionEscala().
  let contenedorEscala: HTMLElement | null = null;

  function valorEfectivo(id: string): string | null {
    for (const fm of filasMontadasActual) {
      const hijo = fm.nodo.hijos.find((h) => h.id === id);

      if (hijo) {
        return hijo.valor_personalizado ?? hijo.valor_defecto;
      }
    }

    return null;
  }

  function escalaActual(): ClaveEscala | "" {
    for (const clave of ["pequeno", "normal", "grande"] as const) {
      const coincide = Object.entries(PRESETS_ESCALA[clave]).every(
        ([id, valor]) => valorEfectivo(id) === valor,
      );

      if (coincide) {
        return clave;
      }
    }

    return "";
  }

  function actualizarSeleccionEscala(): void {
    if (!contenedorEscala) {
      return;
    }

    contenedorEscala.replaceChildren(
      crearGrupoOpciones<ClaveEscala>(
        [
          { texto: "Pequeño", valor: "pequeno" },
          { texto: "Normal", valor: "normal" },
          { texto: "Grande", valor: "grande" },
        ],
        escalaActual() as ClaveEscala,
        (valor) => aplicarEscala(valor),
      ),
    );
  }

  function aplicarEscala(clave: ClaveEscala): void {
    for (const [id, valor] of Object.entries(PRESETS_ESCALA[clave])) {
      for (const fm of filasMontadasActual) {
        const hijo = fm.nodo.hijos.find((h) => h.id === id);

        if (hijo) {
          hijo.valor_personalizado = valor;
          filasConCambio.add(fm.nodo);
          fm.tr.classList.add("configuracion-arbol-editando");
        }
      }
    }

    for (const fm of filasMontadasActual) {
      fm.actualizarColumnas();
    }

    actualizarSeleccionEscala();
  }

  function crearFilaEscala(): HTMLElement {
    const fila = document.createElement("div");
    fila.className = "configuracion-escala-fila";

    const etiqueta = document.createElement("span");
    etiqueta.className = "configuracion-escala-etiqueta";
    etiqueta.textContent = "Escala general";

    contenedorEscala = document.createElement("div");

    fila.append(etiqueta, contenedorEscala);

    return fila;
  }

  // F1: separada de cargar() para poder refrescar la tabla tras
  // "Cargar" tema (Etapa F) sin reiniciar la sesión de apariencia —
  // reiniciarla ahí perdería el preview recién cargado.
  async function recargarTablaApariencia(): Promise<void> {
    const filas = await invoke<FilaCssCruda[]>(
      "configuracion_listar_apariencia",
    );

    const arbolCompleto = construirArbol(filas);

    const coloresTema = construirMapaColoresTema(arbolCompleto);

    const arbol = filtrarArbolPorGrupos(arbolCompleto, gruposPermitidos);

    tbody.innerHTML = "";

    filasConCambio.clear();
    trPorNodo.clear();
    valoresOriginales.clear();

    for (const fila of filas) {
      if (fila.nivel === 0) {
        valoresOriginales.set(fila.id, fila.valor_personalizado);
      }
    }

    const filasMontadas: FilaMontada[] = [];

    for (let i = 0; i < arbol.length; i++) {
      const nodo = arbol[i];

      const { tr, actualizarColumnas } = crearFilaArbol(
        nodo,
        filasMontadas,
        coloresTema,
        filasConCambio,
        valoresOriginales,
      );

      // A4: separación extra para una fila de nivel 2 que tiene una
      // fila hija de nivel 3 debajo (ej. "Separador" → "Fondo de sus
      // filas"), para reforzar la jerarquía de árbol.
      if (nodo.entrada.nivel === 2 && arbol[i + 1]?.entrada.nivel === 3) {
        tr.classList.add("configuracion-arbol-con-hijo");
      }

      filasMontadas.push({ nodo, tr, actualizarColumnas });
      trPorNodo.set(nodo, tr);

      tbody.append(tr);
    }

    filasMontadasActual = filasMontadas;

    actualizarBotonSelectorTema(filas);
    actualizarSeleccionEscala();
  }

  async function cargar(): Promise<void> {
    // E5: reinicia la sesión de apariencia (descarta preview no
    // aplicado de una apertura anterior) y toma nombre/origen del
    // tema realmente aplicado y persistido en disco.
    const sesion = await invoke<{ nombre: string; origen: string }>(
      "configuracion_apariencia_iniciar_sesion",
    );

    nombreTemaSesion = sesion.nombre;
    origenTemaSesion = sesion.origen;
    huboCargaDeTema = false;

    await recargarTablaApariencia();
  }

  function hayEdicionesPendientes(): boolean {
    return filasConCambio.size > 0 || huboCargaDeTema;
  }

  // H5 (corregido): recolecta los hijos de las filas marcadas. Un
  // hijo con valor_personalizado no nulo se manda como cambio normal.
  // Un hijo que volvió a null pero SÍ tenía un valor original
  // guardado (ver valoresOriginales) se manda igual, con
  // SENTINEL_BORRAR como valor — si no se mandara nada, "Guardar
  // cambios" no se entera de que hay que quitar el override
  // persistido y el valor viejo queda vivo en el backend. Un hijo que
  // volvió a null y nunca tuvo valor original no genera ningún
  // cambio (no hay nada que guardar ni que borrar).
  function validarYRecolectar(): {
    cambios: CambioConfiguracion[];
    erroresLocales: string[];
  } {
    const cambios: CambioConfiguracion[] = [];

    for (const nodo of filasConCambio) {
      for (const hijo of nodo.hijos) {
        if (hijo.valor_personalizado !== null) {
          cambios.push({ clave: hijo.id, valor: hijo.valor_personalizado });
        } else if (valoresOriginales.get(hijo.id) !== null) {
          cambios.push({ clave: hijo.id, valor: SENTINEL_BORRAR });
        }
      }
    }

    return { cambios, erroresLocales: [] };
  }

  // H6 (corregido): separa el lote en "a guardar" (valor real) y "a
  // borrar" (SENTINEL_BORRAR) — cada uno va a su comando Rust
  // correspondiente. Si ambos fallan/tienen error, se combinan los
  // resultados; configuracion_restablecer_claves_css no valida ni
  // devuelve errores por clave (solo falla la operación completa si
  // no se puede escribir el archivo), así que cualquier falla ahí se
  // reporta como error general (clave vacía).
  async function aplicarGuardado(
    cambios: CambioConfiguracion[],
  ): Promise<ResultadoGuardado> {
    const aGuardar = cambios.filter(
      (cambio) => cambio.valor !== SENTINEL_BORRAR,
    );
    const aBorrar = cambios
      .filter((cambio) => cambio.valor === SENTINEL_BORRAR)
      .map((cambio) => cambio.clave);

    const errores: ErrorConfiguracion[] = [];

    // Se llama igual con aGuardar vacío cuando huboCargaDeTema: no hay
    // ningún campo puntual tocado, pero igual hay que persistir el
    // tema de sesión completo (y marcarlo como el tema aplicado) —
    // ver aplicar_apariencia en el backend.
    if (aGuardar.length > 0 || huboCargaDeTema) {
      const resultado = await invoke<ResultadoGuardado>(
        "configuracion_guardar_lote_apariencia",
        { cambios: aGuardar },
      );

      errores.push(...resultado.errores);
    }

    if (aBorrar.length > 0 && errores.length === 0) {
      try {
        await invoke("configuracion_restablecer_claves_css", {
          claves: aBorrar,
        });
      } catch (error) {
        errores.push({
          clave: "",
          mensaje: `No se pudo borrar: ${String(error)}`,
        });
      }
    }

    return { errores };
  }

  // H7: ubica, para cada error, el nodo contenedor cuyo hijo tiene ese
  // id — se busca en filasConCambio porque es el único universo de
  // nodos que pudo haber generado un cambio enviado a guardar — y
  // marca su tr vía trPorNodo.
  function marcarErroresGuardado(errores: ErrorConfiguracion[]): void {
    for (const error of errores) {
      const nodo = [...filasConCambio].find((candidato) =>
        candidato.hijos.some((hijo) => hijo.id === error.clave),
      );

      if (!nodo) {
        continue;
      }

      const tr = trPorNodo.get(nodo);
      tr?.classList.add("configuracion-arbol-error");
    }
  }

  // H9
  async function limpiarEstadoTrasGuardado(): Promise<void> {
    filasConCambio.clear();
    huboCargaDeTema = false;

    for (const tr of trPorNodo.values()) {
      tr.classList.remove("configuracion-arbol-editando");
      tr.classList.remove("configuracion-arbol-error");
    }

    await despuesDeAplicar();
  }

  // Restablecer esta pestaña. La pestaña "Tema" (incluirSelectorTema)
  // reinicia la sesión completa (cargar()) y vuelve a mostrar el tema
  // realmente aplicado — no borra los overrides "css." persistidos,
  // esos SON los valores del tema aplicado. La pestaña "Apariencia"
  // (Texto+Dimensiones) no es dueña de esa sesión compartida: solo
  // descarta sus propias ediciones sin guardar releyendo el estado
  // actual (recargarTablaApariencia), sin tocar la sesión de tema que
  // pueda estar en preview desde la otra pestaña.
  async function restablecerPestana(): Promise<void> {
    if (opciones.incluirSelectorTema) {
      await cargar();
    } else {
      await recargarTablaApariencia();
    }

    await despuesDeAplicar();
  }

  // Refresco liviano al entrar a esta pestaña (ver activarTab en
  // vent_configuracion_main.ts): las pestañas "Apariencia" y "Tema"
  // comparten la misma sesión de apariencia en el backend, así que
  // cargar/guardar un tema en una debe reflejarse en la otra. No
  // reinicia la sesión (a diferencia de cargar()) para no perder un
  // preview de tema sin guardar. Se salta si hay ediciones propias
  // pendientes, para no pisarlas.
  async function refrescarDesdeOtraPestana(): Promise<void> {
    if (hayEdicionesPendientes()) {
      return;
    }

    await recargarTablaApariencia();
  }

  if (opciones.incluirEscala) {
    panel.prepend(crearFilaEscala());
  }

  return {
    cargar,
    hayEdicionesPendientes,
    validarYRecolectar,
    aplicarGuardado,
    marcarErroresGuardado,
    limpiarEstadoTrasGuardado,
    restablecerPestana,
    refrescarDesdeOtraPestana,

    // Selector de temas: se monta en la barra de acciones global
    // (ver vent_configuracion_main.ts), no dentro de este panel —
    // solo lo expone la pestaña "Tema".
    elementoBarra: opciones.incluirSelectorTema ? botonSelectorTema : undefined,

    textoConfirmacionRestablecer: opciones.textoConfirmacionRestablecer,
  };
}
