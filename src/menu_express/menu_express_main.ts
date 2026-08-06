// ======================================================
// ⚡🪟 menu_express_Main
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
// ETAPA 6: layout real según datos.forma — Radial (círculo,
// ángulo uniforme 360°/N, radio adaptativo) y Cuadrícula (CSS
// Grid, columnas/filas con la regla "0 = auto": se rellena
// primero la dimensión fija, la otra crece). Tamaños de botón/
// texto en TAMANOS_PX más abajo — mismo valor que menu_express.css
// (config.rs los hará configurables de verdad en la etapa 8).
//
// Todavía sin ejecución real de los botones al hacer clic (llega
// en la etapa 7, junto con Mantenido/Turbo emulados y el cierre
// real Toggle/Efímero tras el up).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import "../styles/styl_variables.css";
import "./menu_express.css";

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
}

// ======================================================
// 🆔 ID DEL MENÚ (query string)
// ======================================================

const id = new URLSearchParams(window.location.search).get("id");

// ======================================================
// 🏗️ ARMAR DOM BASE
// ======================================================

const raiz = document.getElementById("menu-express")!;

const card = document.createElement("div");
card.className = "menu-express-card";
card.setAttribute("data-tauri-drag-region", "");

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

header.append(titulo, botonCerrar);

const cuerpo = document.createElement("div");
cuerpo.className = "menu-express-cuerpo";

card.append(header, cuerpo);
raiz.append(card);

// ======================================================
// 🚪 CERRAR
// ======================================================

function cerrar(): void {
  if (!id) return;

  invoke("cerrar_menu_express", { id }).catch(() => {});
}

botonCerrar.addEventListener("click", cerrar);

// ======================================================
// 🎨 FONDO TEÑIDO CON EL COLOR DE LA FILA
// ------------------------------------------------------
// Mismo vocabulario que la paleta de color de fila
// (--tag-<color> en styl_variables.css). color-mix() para la
// transparencia — el fallback sólido en menu_express.css cubre
// navegadores sin soporte.
// ======================================================

function aplicarColorFondo(color: string): void {
  const variable = color ? `var(--tag-${color})` : "var(--panel2)";

  card.style.setProperty(
    "--menu-color",
    `color-mix(in srgb, ${variable} 55%, rgba(20, 20, 30, 0.85))`,
  );
}

// ======================================================
// 📐 TAMAÑOS EN PX
// ------------------------------------------------------
// Mismo valor que menu_express.css (.tam-*/.txt-*) — acá en TS
// porque el cálculo geométrico (radio del círculo, tamaño de
// grid) necesita el número real, no solo la clase CSS. Única
// fuente de verdad hasta que config.rs los exponga (etapa 8);
// si cambian acá, cambiarlos también en el CSS.
// ======================================================

const TAMANOS_BOTON_PX: Record<
  MenuExpressDatos["tamanoBoton"],
  { ancho: number; alto: number }
> = {
  pequeno: { ancho: 60, alto: 30 },
  mediano: { ancho: 80, alto: 40 },
  grande: { ancho: 100, alto: 50 },
};

// ======================================================
// 🔘 CREAR UN BOTÓN
// ------------------------------------------------------
// Común a los tres layouts — solo cambia cómo se posiciona
// después de creado (grid vs radial vs lista).
// ======================================================

function crearBoton(
  boton: MenuBotonDatos,
  datos: MenuExpressDatos,
): HTMLButtonElement {
  const elemento = document.createElement("button");

  elemento.className = `menu-express-boton tam-${datos.tamanoBoton} txt-${datos.tamanoTexto}`;
  elemento.textContent = boton.renombrar || "(sin nombre)";
  elemento.title = boton.renombrar;

  // Placeholder de esta etapa: la ejecución real (buscar la fila
  // en la caché compilada y llamar a runtime::ejecutar vía
  // menu_express_boton_down/up) llega en la etapa 7. Por ahora
  // solo confirma que el dato de cada botón (filaId) llegó bien
  // hasta acá.
  elemento.addEventListener("click", () => {
    console.log(
      `[MenuExpress] clic en botón (placeholder, sin ejecutar) filaId=${boton.filaId}`,
    );
  });

  return elemento;
}

// ======================================================
// ⭕ LAYOUT RADIAL
// ------------------------------------------------------
// Ángulo uniforme (360°/N), primer botón arriba (-90°) y
// sentido horario. Radio adaptativo: crece con la cantidad de
// botones para que no se encimen, pero nunca baja de un mínimo
// cómodo. Sin límite de botones (spec) — el radio absorbe la
// cantidad, y back_menu_express.rs ya calculó un tamaño de
// ventana acorde (ver calcular_tamano_ventana(), mismo criterio
// espejado en Rust — si uno cambia, cambiar el otro).
// ======================================================

function renderizarRadial(datos: MenuExpressDatos): void {
  const n = datos.botones.length;
  const { ancho, alto } = TAMANOS_BOTON_PX[datos.tamanoBoton];
  const radioBoton = Math.max(ancho, alto) / 2;

  // Mismo cálculo que calcular_tamano_ventana() en
  // back_menu_express.rs — si uno cambia, cambiar el otro.
  const radio = Math.max(
    70,
    (radioBoton + 12) / Math.sin(Math.PI / Math.max(n, 2)),
  );

  if (datos.nombre) {
    const centro = document.createElement("div");
    centro.className = "menu-express-radial-centro";
    centro.textContent = datos.nombre;
    cuerpo.append(centro);
  }

  datos.botones.forEach((boton, indice) => {
    const elemento = crearBoton(boton, datos);

    const anguloGrados = -90 + (360 / n) * indice;
    const anguloRad = (anguloGrados * Math.PI) / 180;

    // Posición en px relativa al centro del contenedor — el
    // contenedor mismo mide (radio*2 + margen) de lado, calculado
    // en calcularDiametroRadial (espejado en back_menu_express.rs),
    // así que "50% + N px" siempre cae dentro.
    const x = radio * Math.cos(anguloRad);
    const y = radio * Math.sin(anguloRad);

    elemento.style.left = `calc(50% + ${x}px)`;
    elemento.style.top = `calc(50% + ${y}px)`;

    cuerpo.append(elemento);
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
// 🔘 RENDERIZAR BOTONES
// ------------------------------------------------------
// Despacha según datos.forma. La clase forma-* en .cuerpo activa
// el CSS correspondiente (grid / posicionamiento absoluto) —
// ver menu_express.css.
// ======================================================

function renderizarBotones(datos: MenuExpressDatos): void {
  cuerpo.innerHTML = "";
  cuerpo.style.gridTemplateColumns = "";
  cuerpo.style.gridTemplateRows = "";
  cuerpo.classList.remove("forma-radial", "forma-cuadricula");

  if (datos.botones.length === 0) {
    const vacio = document.createElement("div");
    vacio.className = "menu-express-vacio";
    vacio.textContent = "Este menú no tiene botones.";
    cuerpo.append(vacio);
    return;
  }

  if (datos.forma === "radial") {
    cuerpo.classList.add("forma-radial");
    renderizarRadial(datos);
    return;
  }

  if (datos.forma === "cuadricula") {
    cuerpo.classList.add("forma-cuadricula");
    renderizarCuadricula(datos);
    return;
  }

  renderizarLista(datos);
}

// ======================================================
// 🏁 INICIAR
// ======================================================

async function iniciar(): Promise<void> {
  if (!id) {
    titulo.textContent = "Menú";
    cuerpo.innerHTML = "";
    const error = document.createElement("div");
    error.className = "menu-express-vacio";
    error.textContent = "Falta el id del menú en la URL.";
    cuerpo.append(error);
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
    titulo.textContent = "Menú";
    cuerpo.innerHTML = "";
    const error = document.createElement("div");
    error.className = "menu-express-vacio";
    error.textContent = "Este menú ya no está disponible.";
    cuerpo.append(error);
    return;
  }

  titulo.textContent = datos.nombre || "Menú";
  aplicarColorFondo(datos.color);
  renderizarBotones(datos);
}

iniciar();
