// ======================================================
// ⚡🪟 vent_menu_express_Main
// ------------------------------------------------------
// Punto de entrada de la ventana flotante de MenuExpress
// (menu_express.html — página independiente, ver
// vite.config.ts). El id del menú a mostrar viaja en la URL
// (?id=...) — back_menu_express.rs arma esa URL al crear la
// ventana (ver crear_ventana()).
//
// Los datos (nombre/botones/forma/etc.) se leen UNA sola vez
// al cargar, vía obtener_datos_menu_express — mismo patrón que
// captura_main.ts con obtener_config_captura_activa: el
// backend ya dejó los datos listos ANTES de crear la ventana
// (back_menu_express::abrir_o_alternar registra antes de
// llamar a crear_ventana), sin carrera posible.
//
// ETAPA 6: layout real según datos.forma — Radial (anillo de
// gajos, ver más abajo) y Cuadrícula (CSS Grid, columnas/filas
// con la regla "0 = auto": se rellena primero la dimensión fija,
// la otra crece).
//
// ETAPA 7: ejecución real de los botones — mousedown manda el down
// real (menu_express_boton_down), mouseup el up real
// (menu_express_boton_up), y éste último resuelve Comportamiento
// (Toggle/Efímero) del lado de Rust. Mantenido/Turbo se emulan solos:
// el tiempo real entre down y up (mientras el botón del mouse siga
// presionado) es exactamente lo que runtime.rs ya sabe interpretar
// (mismo motor que un trigger físico sostenido, ver back_menu_express.rs).
//
// ETAPA 8: tamaños de botón/texto ya no están hardcodeados — se leen
// de config.rs vía obtener_tamanos_menu_express (ver
// leerTamanosMenuExpress más abajo), única fuente de verdad real.
// ubicacion "Persistente" ahora recuerda la última posición real
// (ver back_menu_express.rs) en vez de un punto fijo. Comportamiento
// Efímero ahora cierra con fade-out (ver cerrarConFade más abajo) en
// vez de destruirse en seco.
//
// ETAPA 9: Radial dejó de ser botones cuadrados repartidos en un
// círculo — ahora es un ANILLO CONTINUO de gajos (trapecio
// redondeado, como un gráfico de torta con hueco central), sin
// barra superior. El hueco central muestra el nombre del menú + el
// botón de cerrar (reemplaza al header, que en Radial ya no existe).
// Como los gajos cubren el anillo completo (no dependen de no
// solaparse entre sí), el radio YA NO crece con la cantidad de
// botones (a diferencia del sistema viejo) — depende solo del
// tamaño de botón elegido (menu_extra.tamanoBoton), igual criterio
// espejado en back_menu_express.rs::calcular_tamano_ventana. Cada
// gajo se recorta con clip-path:path(...) usando un arco SVG — el
// hit-test del click respeta ese recorte (Chromium/WebView2), así
// que gajos vecinos no se pisan el área de clic aunque cada
// elemento ocupe, de fondo, todo el anillo.
// Cuadrícula NO cambia — sigue con su barra superior de siempre.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { aplicarOverridesApariencia } from "../core/core_apariencia";

import "../styles/styl_variables.css";
import "../styles/styl_menu_express.css";

import { iniciarAjusteTextoBotones } from "../util/util_texto_boton";

void aplicarOverridesApariencia();

// ======================================================
// 🧭 TIPOS
// ------------------------------------------------------
// Espejo de MenuExpressDatosUI / MenuBotonUI en
// back_menu_express.rs (mismo vocabulario string que
// core_menu_express.ts).
// ======================================================

interface MenuBotonDatos {
  filaId: string;

  renombrar: string;

  // Color de la fila REFERENCIADA (back_menu_express.rs ya la
  // resuelve al compilar) — "" si esa fila no tiene color asignado.
  // Solo se usa como borde cuando datos.colorBoton === "color".
  color: string;
}

interface MenuExpressDatos {
  nombre: string;

  botones: MenuBotonDatos[];

  forma: "radial" | "cuadricula";

  columnas: number;

  filas: number;

  comportamiento: "toggle" | "efimero";

  ubicacion: "persistente" | "cursor";

  tamanoBoton: "pequeno" | "mediano" | "grande";

  tamanoTexto: "pequeno" | "mediano" | "grande";

  color: string;

  // Nueva variable global (pulido, punto "Color botón"): "color" |
  // "monocromo" — ver ColorBotonMenu en core_menu_express.ts.
  colorBoton: "color" | "monocromo";
}

// ======================================================
// 🆔 ID DEL MENÚ (query string)
// ======================================================

const id = new URLSearchParams(window.location.search).get("id");

// ======================================================
// 🏗️ RAÍZ + ESTADO DE LA ESTRUCTURA ACTUAL
// ------------------------------------------------------
// A diferencia de antes, el DOM ya NO se arma entero al cargar el
// módulo: Cuadrícula tiene header y Radial no, así que hace falta
// saber `datos.forma` antes de decidir qué estructura construir
// (ver construirEstructuraConHeader / construirEstructuraRadial).
// `card` y `cuerpo` quedan como referencias mutables al elemento
// vigente — cerrar()/cerrarConFade() siempre operan sobre la
// estructura que esté montada en ese momento.
// ======================================================

const raiz = document.getElementById("menu-express")!;

let card: HTMLDivElement;
let cuerpo: HTMLDivElement;

// ======================================================
// 🚪 CERRAR
// ======================================================

function cerrar(): void {
  if (!id) return;

  invoke("cerrar_menu_express", { id }).catch(() => {});
}

// ======================================================
// 🌫️ CIERRE SUAVE (Comportamiento Efímero) — ETAPA 8
// ------------------------------------------------------
// back_menu_express.rs::boton_up ya NO cierra la ventana: solo
// avisa (true) que este menú es Efímero y debe cerrarse tras el
// clic. Acá se juega la animación (clase "cerrando", ver
// menu_express.css) y recién cuando termina se invoca el cierre
// real — mismo comando que usa el botón [x]. DURACION_FADE_MS
// tiene que calzar con la transición CSS de .menu-express-card
// (0.18s) para no cortar la animación a mitad de camino.
// ======================================================

const DURACION_FADE_MS = 180;

function cerrarConFade(): void {
  card.classList.add("cerrando");
  window.setTimeout(cerrar, DURACION_FADE_MS);
}

// ======================================================
// 🎨 FONDO TEÑIDO CON EL COLOR DE LA FILA
// ------------------------------------------------------
// Mismo vocabulario que la paleta de color de fila
// (--tag-<color> en styl_variables.css). color-mix() para la
// transparencia — el fallback sólido en menu_express.css cubre
// navegadores sin soporte. En Radial esta variable la usan los
// gajos (ver .menu-express-boton--gajo en menu_express.css); en
// Cuadrícula la usa la tarjeta entera, igual que antes.
// ======================================================

function aplicarColorFondo(color: string): void {
  const variable = color ? `var(--tag-${color})` : "var(--panel2)";

  card.style.setProperty(
    "--menu-color",
    `color-mix(in srgb, ${variable} 45%, color-mix(in srgb, var(--overlay-superficie) 60%, transparent))`,
  );

  card.style.setProperty(
    "--menu-gradient",
    `radial-gradient(circle at 50% 30%, color-mix(in srgb, ${variable} 60%, rgba(30, 30, 45, 0.6)), color-mix(in srgb, ${variable} 25%, rgba(10, 10, 18, 0.9)))`,
  );
}

// ======================================================
// 🎨 BORDE DE COLOR POR BOTÓN — "Color Botón" (pulido)
// ------------------------------------------------------
// Monocromo (datos.colorBoton === "monocromo", default): no hace
// nada — el botón/gajo se queda con el borde/color heredado de
// siempre (el de la ventana, ver aplicarColorFondo). Color: cada
// botón toma como borde el --tag-<color> de SU PROPIA fila
// referenciada (boton.color, ya resuelto por back_menu_express.rs).
// Si esa fila no tiene color asignado (boton.color === ""), NO se
// toca el borde — se mantiene heredado de la ventana (mismo
// resultado visual que Monocromo para ese botón puntual, spec).
//
// Se aplica sobre --boton-color-borde en vez de tocar `border`
// directo, para que el mismo valor sirva tanto al borde real de
// Cuadrícula/lista (.menu-express-boton) como al filtro/contorno del
// gajo en Radial (que no tiene un "border" geométrico tradicional,
// ver .menu-express-boton--gajo en menu_express.css).
// ======================================================

function aplicarColorBorde(
  elemento: HTMLElement,
  colorBoton: MenuExpressDatos["colorBoton"],
  colorFila: string,
): void {
  if (colorBoton !== "color" || !colorFila) return;

  elemento.style.setProperty("--boton-color-borde", `var(--tag-${colorFila})`);
  elemento.classList.add("menu-express-boton--color-propio");
}

// ======================================================
// 📐 TAMAÑOS EN PX (config.rs — etapa 8)
// ------------------------------------------------------
// Se leen una sola vez al iniciar, vía obtener_tamanos_menu_express
// (ver config.rs / comandos.rs) — config.rs es la única fuente de
// verdad real y configurable; las clases .tam-*/.txt-* en
// menu_express.css quedan solo como valor de respaldo (por si el
// comando fallara) — el valor real efectivo siempre se aplica
// inline (style.width/height/fontSize), que gana sobre la clase.
// ======================================================

interface MenuExpressTamanos {
  botonPequeno: { ancho: number; alto: number };
  botonMediano: { ancho: number; alto: number };
  botonGrande: { ancho: number; alto: number };
  textoPequeno: number;
  textoMediano: number;
  textoGrande: number;
}

// Respaldo (mismo valor que .tam-*/.txt-* en menu_express.css) —
// solo se usa si obtener_tamanos_menu_express llegara a fallar.
let TAMANOS: MenuExpressTamanos = {
  botonPequeno: { ancho: 60, alto: 30 },
  botonMediano: { ancho: 80, alto: 40 },
  botonGrande: { ancho: 100, alto: 50 },
  textoPequeno: 10,
  textoMediano: 13,
  textoGrande: 16,
};

function tamanoBotonPx(tamano: MenuExpressDatos["tamanoBoton"]): {
  ancho: number;
  alto: number;
} {
  if (tamano === "pequeno") return TAMANOS.botonPequeno;
  if (tamano === "grande") return TAMANOS.botonGrande;
  return TAMANOS.botonMediano;
}

function tamanoTextoPx(tamano: MenuExpressDatos["tamanoTexto"]): number {
  if (tamano === "pequeno") return TAMANOS.textoPequeno;
  if (tamano === "grande") return TAMANOS.textoGrande;
  return TAMANOS.textoMediano;
}

async function leerTamanosMenuExpress(): Promise<void> {
  try {
    const resultado = await invoke<{
      botonPequeno: [number, number];
      botonMediano: [number, number];
      botonGrande: [number, number];
      textoPequeno: number;
      textoMediano: number;
      textoGrande: number;
    }>("obtener_tamanos_menu_express");

    TAMANOS = {
      botonPequeno: {
        ancho: resultado.botonPequeno[0],
        alto: resultado.botonPequeno[1],
      },
      botonMediano: {
        ancho: resultado.botonMediano[0],
        alto: resultado.botonMediano[1],
      },
      botonGrande: {
        ancho: resultado.botonGrande[0],
        alto: resultado.botonGrande[1],
      },
      textoPequeno: resultado.textoPequeno,
      textoMediano: resultado.textoMediano,
      textoGrande: resultado.textoGrande,
    };
  } catch {
    // Se queda con el respaldo de arriba.
  }
}

// ======================================================
// 🏗️ ESTRUCTURA CON HEADER (Cuadrícula / estados de error)
// ------------------------------------------------------
// Igual que la única estructura que existía antes de la etapa 9.
// Se sigue usando para Cuadrícula y para los estados en los que
// todavía no se sabe `datos.forma` (falta id / datos no
// disponibles) — en esos casos no hay nada mejor que mostrar que
// un cartel adentro de la tarjeta de siempre.
// ======================================================

function construirEstructuraConHeader(): {
  titulo: HTMLSpanElement;
  cuerpoNuevo: HTMLDivElement;
} {
  raiz.innerHTML = "";

  const nuevaCard = document.createElement("div");
  nuevaCard.className = "menu-express-card";
  nuevaCard.setAttribute("data-tauri-drag-region", "");

  const header = document.createElement("div");
  header.className = "menu-express-header";
  header.setAttribute("data-tauri-drag-region", "");

  const titulo = document.createElement("span");
  titulo.className = "menu-express-titulo";
  titulo.setAttribute("data-tauri-drag-region", "");

  const botonCerrar = document.createElement("button");
  botonCerrar.className = "menu-express-cerrar";
  botonCerrar.textContent = "×";
  botonCerrar.title = "Cerrar";
  botonCerrar.addEventListener("click", cerrar);

  header.append(titulo, botonCerrar);

  const cuerpoNuevo = document.createElement("div");
  cuerpoNuevo.className = "menu-express-cuerpo";
  // Permite arrastrar la ventana desde cualquier zona vacía del
  // cuerpo (no solo la barra de título) — Tauri arma la región de
  // arrastre mirando el elemento puntual donde ocurre el mousedown,
  // así que los botones (sin este atributo) siguen funcionando
  // como clics normales sin arrastrar la ventana.
  cuerpoNuevo.setAttribute("data-tauri-drag-region", "");

  nuevaCard.append(header, cuerpoNuevo);
  raiz.append(nuevaCard);

  card = nuevaCard;
  cuerpo = cuerpoNuevo;

  return { titulo, cuerpoNuevo };
}

// ======================================================
// 🏗️ ESTRUCTURA RADIAL (sin header) — ETAPA 9
// ------------------------------------------------------
// Sin barra superior: la tarjeta es transparente (solo se ven los
// gajos + el círculo central), y el círculo central reemplaza al
// header — muestra el nombre del menú y el botón de cerrar. El
// arrastre de la ventana queda limitado al círculo central (spec:
// "solo desde el centro"), nunca desde el anillo de gajos (esa zona
// tiene que quedar libre para clickear los gajos sin arrastrar por
// error).
// ======================================================

function construirEstructuraRadial(): { centroTitulo: HTMLSpanElement } {
  raiz.innerHTML = "";

  const nuevaCard = document.createElement("div");
  nuevaCard.className = "menu-express-card menu-express-card--radial";

  const cuerpoNuevo = document.createElement("div");
  cuerpoNuevo.className = "menu-express-cuerpo forma-radial";

  const centro = document.createElement("div");
  centro.className = "menu-express-radial-centro";
  centro.setAttribute("data-tauri-drag-region", "");

  const centroTitulo = document.createElement("span");
  centroTitulo.className = "menu-express-radial-centro-titulo";

  const botonCerrar = document.createElement("button");
  botonCerrar.className = "menu-express-cerrar menu-express-radial-cerrar";
  botonCerrar.textContent = "×";
  botonCerrar.title = "Cerrar";
  botonCerrar.addEventListener("click", cerrar);

  centro.append(centroTitulo, botonCerrar);
  cuerpoNuevo.append(centro);
  nuevaCard.append(cuerpoNuevo);
  raiz.append(nuevaCard);

  card = nuevaCard;
  cuerpo = cuerpoNuevo;

  return { centroTitulo };
}

// ======================================================
// 🔘 EVENTOS COMUNES DE UN BOTÓN (down/up real)
// ------------------------------------------------------
// Común a los tres layouts (lista, cuadrícula, gajo radial) —
// separado de la creación del elemento en sí para poder reusarlo
// tal cual con la forma de gajo (ETAPA 9), que ya no crea el botón
// desde cero en el mismo lugar donde antes vivía este código.
//
// mousedown/mouseup (NO click): un click dispara recién al
// soltar, pero acá el down y el up son eventos DISTINTOS que
// Runtime necesita por separado — el tiempo real entre uno y
// otro es lo que define cuánto dura un Mantenido o cuántas
// vueltas alcanza a dar un Turbo (ver back_menu_express.rs,
// boton_down/boton_up).
//
// El mouseup se escucha en `document`, no en el propio botón:
// si el usuario suelta el botón del mouse afuera del elemento
// (arrastró el cursor antes de soltar), igual tiene que llegar
// el up real — soltar "en cualquier lado" es análogo a soltar
// una tecla física, no depende de seguir con el cursor encima.
// Se limpia el listener después de cada uso (una sola vez por
// down) para no ir acumulando listeners de botones ya sueltos.
// ======================================================

function adjuntarEventosBoton(
  elemento: HTMLButtonElement,
  boton: MenuBotonDatos,
): void {
  elemento.addEventListener("mousedown", (evento) => {
    // Solo botón izquierdo — clic derecho/medio no ejecuta nada
    // (mismo criterio que "Click izquierdo solo" bloqueado como
    // trigger de MenuExpress, ver spec etapa 8).
    if (evento.button !== 0) return;

    evento.preventDefault();

    elemento.classList.add("presionado");

    invoke("menu_express_boton_down", { filaId: boton.filaId }).catch(() => {});

    const soltar = async (subida: MouseEvent): Promise<void> => {
      if (subida.button !== 0) return;

      document.removeEventListener("mouseup", soltar);
      elemento.classList.remove("presionado");

      if (!id) return;

      let esEfimero = false;

      try {
        esEfimero = await invoke<boolean>("menu_express_boton_up", {
          idMenu: id,
          filaId: boton.filaId,
        });
      } catch {
        esEfimero = false;
      }

      // Efímero: la ventana sigue existiendo (Rust ya no la cierra
      // sola, ver back_menu_express.rs) — se juega el fade-out acá
      // y recién al terminar se invoca el cierre real.
      if (esEfimero) cerrarConFade();
    };

    document.addEventListener("mouseup", soltar);
  });
}

// ======================================================
// 🔘 CREAR UN BOTÓN (lista / cuadrícula)
// ======================================================

function crearBoton(
  boton: MenuBotonDatos,
  datos: MenuExpressDatos,
): HTMLButtonElement {
  const elemento = document.createElement("button");

  elemento.className = `menu-express-boton tam-${datos.tamanoBoton} txt-${datos.tamanoTexto}`;
  elemento.textContent = boton.renombrar || "(sin nombre)";
  elemento.title = boton.renombrar;

  // Valor real efectivo (config.rs) aplicado inline — gana sobre
  // la clase .tam-*/.txt-* de respaldo. En modo lista/cuadrícula
  // el ancho lo pisa el propio layout (100%/celda de grid) — ver
  // menu_express.css.
  const { alto } = tamanoBotonPx(datos.tamanoBoton);
  elemento.style.height = `${alto}px`;
  elemento.style.fontSize = `${tamanoTextoPx(datos.tamanoTexto)}px`;

  aplicarColorBorde(elemento, datos.colorBoton, boton.color);

  adjuntarEventosBoton(elemento, boton);

  return elemento;
}

// ======================================================
// ⭕ LAYOUT RADIAL — ANILLO DE GAJOS (ETAPA 9)
// ------------------------------------------------------
// Cada botón es un gajo (trapecio redondeado) recortado con
// clip-path:path(...) sobre un arco SVG — el elemento ocupa, de
// fondo, todo el anillo (mismo tamaño que .menu-express-cuerpo),
// pero solo se ve/clickea la porción dentro de su ángulo. El
// primer gajo arranca arriba (-90°) y avanza en sentido horario,
// mismo criterio que el sistema anterior. GAP_GRADOS separa cada
// gajo de su vecino (recorte simétrico a cada lado del ángulo) para
// que los colores de fila (a futuro, variable "Color botón") no se
// toquen entre sí.
//
// El radio NO depende de la cantidad de botones (a diferencia del
// sistema anterior): como los gajos cubren el anillo completo, más
// botones simplemente angostan cada gajo — nunca se solapan. El
// tamaño del anillo depende solo de menu_extra.tamanoBoton, mismo
// criterio espejado en back_menu_express.rs::calcular_tamano_ventana
// (calcularRadiosRadial de acá == esa función de allá — si uno
// cambia, cambiar el otro).
// ======================================================

const RADIAL_GAP_GRADOS = 3;

interface RadiosRadial {
  huecoRadio: number;
  grosor: number;
  radioExterior: number;
  diametro: number;
}

function calcularRadiosRadial(datos: MenuExpressDatos): RadiosRadial {
  const { alto } = tamanoBotonPx(datos.tamanoBoton);

  const grosor = alto;
  const huecoRadio = Math.round(alto * 1.3);
  const radioExterior = huecoRadio + grosor;
  const diametro = radioExterior * 2 + 16;

  return { huecoRadio, grosor, radioExterior, diametro };
}

function puntoEnAngulo(
  cx: number,
  cy: number,
  radio: number,
  anguloGrados: number,
): { x: number; y: number } {
  const anguloRad = (anguloGrados * Math.PI) / 180;

  return {
    x: cx + radio * Math.cos(anguloRad),
    y: cy + radio * Math.sin(anguloRad),
  };
}

// Path del gajo: arco exterior (sentido horario, sweep=1) + línea al
// borde interior + arco interior de vuelta (sentido antihorario,
// sweep=0) + cierre. Coordenadas en el mismo espacio de px que el
// contenedor (diametro x diametro) — clip-path:path() usa la caja
// del propio elemento como sistema de referencia.
function pathGajo(
  cx: number,
  cy: number,
  rInterior: number,
  rExterior: number,
  anguloInicio: number,
  anguloFin: number,
): string {
  const arcoGrande = anguloFin - anguloInicio > 180 ? 1 : 0;

  const outerA = puntoEnAngulo(cx, cy, rExterior, anguloInicio);
  const outerB = puntoEnAngulo(cx, cy, rExterior, anguloFin);
  const innerB = puntoEnAngulo(cx, cy, rInterior, anguloFin);
  const innerA = puntoEnAngulo(cx, cy, rInterior, anguloInicio);

  return (
    `M ${outerA.x} ${outerA.y} ` +
    `A ${rExterior} ${rExterior} 0 ${arcoGrande} 1 ${outerB.x} ${outerB.y} ` +
    `L ${innerB.x} ${innerB.y} ` +
    `A ${rInterior} ${rInterior} 0 ${arcoGrande} 0 ${innerA.x} ${innerA.y} Z`
  );
}

function crearGajo(
  boton: MenuBotonDatos,
  datos: MenuExpressDatos,
  radios: RadiosRadial,
  anguloInicio: number,
  anguloFin: number,
): HTMLButtonElement {
  const elemento = document.createElement("button");
  elemento.className = `menu-express-boton menu-express-boton--gajo txt-${datos.tamanoTexto}`;
  elemento.title = boton.renombrar;

  const cx = radios.diametro / 2;
  const cy = radios.diametro / 2;

  const path = pathGajo(
    cx,
    cy,
    radios.huecoRadio,
    radios.radioExterior,
    anguloInicio,
    anguloFin,
  );

  elemento.style.width = `${radios.diametro}px`;
  elemento.style.height = `${radios.diametro}px`;
  elemento.style.clipPath = `path("${path}")`;

  // Guardar el path en variable CSS para el borde
  elemento.style.setProperty("--gajo-path", `"${path}"`);

  // SVG para el borde (mismo path, stroke)
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute(
    "style",
    `
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    overflow: visible;
  `,
  );
  svg.setAttribute("viewBox", `0 0 ${radios.diametro} ${radios.diametro}`);

  const pathEl = document.createElementNS("http://www.w3.org/2000/svg", "path");
  pathEl.setAttribute("d", path);
  pathEl.setAttribute("fill", "none");
  pathEl.setAttribute("stroke", "var(--boton-color-borde, transparent)");
  pathEl.setAttribute("stroke-width", "2");
  pathEl.setAttribute("stroke-linejoin", "round");
  pathEl.setAttribute("stroke-linecap", "round");
  svg.append(pathEl);
  elemento.append(svg);

  // Etiqueta del gajo
  const anguloMedio = (anguloInicio + anguloFin) / 2;
  const radioMedio = (radios.huecoRadio + radios.radioExterior) / 2;
  const puntoTexto = puntoEnAngulo(cx, cy, radioMedio, anguloMedio);

  const etiqueta = document.createElement("span");
  etiqueta.className = "menu-express-boton-etiqueta";
  etiqueta.textContent = boton.renombrar || "(sin nombre)";
  etiqueta.style.left = `${puntoTexto.x}px`;
  etiqueta.style.top = `${puntoTexto.y}px`;
  etiqueta.style.fontSize = `${tamanoTextoPx(datos.tamanoTexto)}px`;
  etiqueta.style.maxWidth = `${radios.grosor - 8}px`;

  elemento.append(etiqueta);

  // Aplicar color de borde (clase + variable CSS)
  if (datos.colorBoton === "color" && boton.color) {
    elemento.style.setProperty(
      "--boton-color-borde",
      `var(--tag-${boton.color})`,
    );
    elemento.classList.add("menu-express-boton--color-propio");
  }

  adjuntarEventosBoton(elemento, boton);

  return elemento;
}

function renderizarRadial(
  datos: MenuExpressDatos,
  centroTitulo: HTMLSpanElement,
): void {
  centroTitulo.textContent = datos.nombre || "Menú";

  const radios = calcularRadiosRadial(datos);

  cuerpo.style.width = `${radios.diametro}px`;
  cuerpo.style.height = `${radios.diametro}px`;

  const centro = cuerpo.querySelector<HTMLDivElement>(
    ".menu-express-radial-centro",
  );
  if (centro) {
    centro.style.width = `${radios.huecoRadio * 2}px`;
    centro.style.height = `${radios.huecoRadio * 2}px`;
  }

  const n = datos.botones.length;

  if (n === 0) return;

  const anguloBoton = 360 / n;
  const mitadGap = RADIAL_GAP_GRADOS / 2;

  datos.botones.forEach((boton, indice) => {
    const base = -90 + anguloBoton * indice;
    const gajo = crearGajo(
      boton,
      datos,
      radios,
      base + mitadGap,
      base + anguloBoton - mitadGap,
    );

    // Los gajos se insertan ANTES del centro (que ya está en el
    // DOM) para que el círculo central quede siempre por encima
    // visualmente y reciba los clics de arrastre/cerrar sin que un
    // gajo vecino se los tape.
    cuerpo.insertBefore(gajo, cuerpo.firstChild);
  });
}

// ======================================================
// ▦ LAYOUT CUADRÍCULA
// ------------------------------------------------------
// Regla (spec): solo una de columnas/filas puede limitar a la
// vez — la que vale 0 es la flexible y se acomoda al número de
// botones. Se rellena primero la dimensión fija (ej. 5 filas con
// 10 botones → 2 columnas). Si ambas son 0 (no debería pasar,
// crearMenuExtra() siempre deja una fija — pero por si acaso) o
// algún valor no es válido, se toma como 1 (spec).
// ======================================================

function calcularGrid(
  n: number,
  columnas: number,
  filas: number,
): { columnas: number; filas: number } {
  const col =
    Number.isFinite(columnas) && columnas > 0 ? Math.floor(columnas) : 0;
  const fil = Number.isFinite(filas) && filas > 0 ? Math.floor(filas) : 0;

  if (fil > 0) {
    // Filas fijas → columnas se acomoda (rellena filas primero).
    return { columnas: Math.max(1, Math.ceil(n / fil)), filas: fil };
  }

  if (col > 0) {
    // Columnas fijas → filas se acomoda.
    return { columnas: col, filas: Math.max(1, Math.ceil(n / col)) };
  }

  // Ninguna de las dos es válida: ambas "1" (spec: valor no
  // válido → 1), lo que en la práctica cae a una columna vertical.
  return { columnas: 1, filas: Math.max(1, n) };
}

function renderizarCuadricula(datos: MenuExpressDatos): void {
  const n = datos.botones.length;
  const { columnas, filas } = calcularGrid(n, datos.columnas, datos.filas);

  cuerpo.classList.add("forma-cuadricula");
  cuerpo.style.gridTemplateColumns = `repeat(${columnas}, 1fr)`;
  cuerpo.style.gridTemplateRows = `repeat(${filas}, 1fr)`;

  datos.botones.forEach((boton) => {
    cuerpo.append(crearBoton(boton, datos));
  });
}

// ======================================================
// 📋 LAYOUT LISTA (fallback)
// ------------------------------------------------------
// No debería alcanzarse — datos.forma siempre es "radial" o
// "cuadricula" (ver core_menu_express.ts) — pero queda como
// respaldo defensivo si llegara un valor inesperado.
// ======================================================

function renderizarLista(datos: MenuExpressDatos): void {
  datos.botones.forEach((boton) => {
    cuerpo.append(crearBoton(boton, datos));
  });
}

// ======================================================
// 🏁 INICIAR
// ======================================================

async function iniciar(): Promise<void> {
  await leerTamanosMenuExpress();

  if (!id) {
    const { titulo, cuerpoNuevo } = construirEstructuraConHeader();
    titulo.textContent = "Menú";
    const error = document.createElement("div");
    error.className = "menu-express-vacio";
    error.textContent = "Falta el id del menú en la URL.";
    cuerpoNuevo.append(error);
    return;
  }

  let datos: MenuExpressDatos | null;

  try {
    datos = await invoke<MenuExpressDatos | null>(
      "obtener_datos_menu_express",
      { id },
    );
  } catch {
    datos = null;
  }

  // No debería pasar (back_menu_express registra los datos ANTES de
  // crear esta ventana) — pero si el menú ya se cerró/recompiló
  // justo en el medio, no hay nada coherente que mostrar.
  if (!datos) {
    const { titulo, cuerpoNuevo } = construirEstructuraConHeader();
    titulo.textContent = "Menú";
    const error = document.createElement("div");
    error.className = "menu-express-vacio";
    error.textContent = "Este menú ya no está disponible.";
    cuerpoNuevo.append(error);
    return;
  }

  if (datos.forma === "radial") {
    const { centroTitulo } = construirEstructuraRadial();
    aplicarColorFondo(datos.color);

    if (datos.botones.length === 0) {
      const vacio = document.createElement("div");
      vacio.className = "menu-express-vacio menu-express-vacio--radial";
      vacio.textContent = "Este menú no tiene botones.";
      cuerpo.append(vacio);
    }

    renderizarRadial(datos, centroTitulo);
    return;
  }

  const { titulo, cuerpoNuevo } = construirEstructuraConHeader();
  titulo.textContent = datos.nombre || "Menú";
  aplicarColorFondo(datos.color);

  if (datos.botones.length === 0) {
    const vacio = document.createElement("div");
    vacio.className = "menu-express-vacio";
    vacio.textContent = "Este menú no tiene botones.";
    cuerpoNuevo.append(vacio);
    return;
  }

  if (datos.forma === "cuadricula") {
    renderizarCuadricula(datos);
    return;
  }

  renderizarLista(datos);
}

iniciarAjusteTextoBotones();
iniciar();
