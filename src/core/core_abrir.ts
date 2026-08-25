// ======================================================
// 📂 core_Abrir
// ------------------------------------------------------
// Modelo del tipo "Abrir Archivo/App" (filaPerfil.tipo === "abrir").
// Igual criterio que MenuExpress/Portapapeles: dos objetos siempre
// presentes en la fila (abrirAccion / abrirExtra), con efecto solo
// cuando tipo === "abrir". A diferencia de esos dos, acá no hay id
// propio ni pool compartido — cada fila es totalmente independiente,
// dueña de su propia ruta, sin nada que coordinar con otras filas.
//
// Espejo exacto de AbrirAccionJson / AbrirExtraJson en
// perfil_json.rs (AbrirExtraJson viaja en camelCase vía
// #[serde(rename_all = "camelCase")] del lado Rust) — viaja tal cual
// hacia Rust en compilar_perfil, sin traducción adicional.
// ======================================================

export type IniciarAbrir = "ventana" | "minimizado" | "maximizado";

export type InstanciasAbrir = "unica" | "multiple";

// ======================================================
// 🎯 ACCIÓN (columna Acción)
// ------------------------------------------------------
// Único campo: la ruta absoluta elegida con "Seleccionar..." —
// archivo, carpeta, .exe o .lnk. null hasta que se elige algo (mismo
// criterio de "dato faltante" que el resto del compilador — la fila
// se descarta en silencio mientras no haya ruta, ver compilador.rs).
// ======================================================

export interface AbrirAccionPerfil {
  ruta: string | null;
}

// ======================================================
// 🎛️ EXTRA (columna Extra)
// ------------------------------------------------------
// abrirCon: ruta absoluta de un programa alternativo elegido para
//   abrir el archivo (en vez del asociado por Windows). Solo tiene
//   sentido cuando ruta NO es un .exe/.lnk — null si no se
//   personalizó (se usa el programa por defecto del sistema).
// argumento: texto libre agregado a la ejecución cuando ruta ES un
//   .exe (ej. "--config"). "" si no se personalizó.
// abrirCon y argumento son mutuamente excluyentes EN LA UI (cuál de
// los dos se muestra depende de esRutaExe(abrirAccion.ruta), ver
// comp_popup_abrir.ts) — pero ambos campos viajan siempre presentes
// acá, igual que en AbrirExtraJson.
// ======================================================

export interface AbrirExtraPerfil {
  iniciar: IniciarAbrir;

  instancias: InstanciasAbrir;

  abrirCon: string | null;

  argumento: string;
}

// ======================================================
// ➕ CREAR ACCIÓN / EXTRA POR DEFECTO
// ======================================================

export function crearAbrirAccion(): AbrirAccionPerfil {
  return {
    ruta: null,
  };
}

export function crearAbrirExtra(): AbrirExtraPerfil {
  return {
    iniciar: "ventana",

    instancias: "multiple",

    abrirCon: null,

    argumento: "",
  };
}

// ======================================================
// 🔎 ¿ES UN .exe?
// ------------------------------------------------------
// Determina qué campo de AbrirExtraPerfil corresponde mostrar en el
// popup Extra ("Abrir con..." vs "Abrir con Argumento...", ver
// comp_popup_abrir.ts) y refleja el mismo criterio que usa
// runtime.rs (Rust) al ejecutar, vía su propia extension_de(). null
// o cadena vacía → false.
// ======================================================

export function esRutaExe(ruta: string | null): boolean {
  if (!ruta) {
    return false;
  }

  return ruta.toLowerCase().endsWith(".exe");
}

// ======================================================
// 🔎 EXTENSIÓN DE UNA RUTA
// ------------------------------------------------------
// Usada por el popup "Abrir con..." (comp_popup_abrir_con.ts, Etapa
// 11) para pedirle a obtener_programas_abrir_con() los recientes de
// esa extensión puntual (ej. abrirAccion.ruta = "...\\foto.jpg" →
// "jpg"). "" si no hay ruta o no tiene extensión (carpetas) — el
// backend interpreta una extensión vacía devolviendo solo los
// instalados, sin recientes.
// ======================================================

export function extensionDeRuta(ruta: string | null): string {
  if (!ruta) {
    return "";
  }

  const nombre = nombreDeRuta(ruta);

  const punto = nombre.lastIndexOf(".");

  if (punto <= 0) {
    return "";
  }

  return nombre.slice(punto + 1).toLowerCase();
}

// ======================================================
// 📝 TOOLTIP DEL BOTÓN EXTRA (columna Extra, ícono ∴)
// ------------------------------------------------------
// Lista de líneas "Subtítulo: Elección" — todos los campos de
// abrirExtra. "Argumento" o "Abrir con" según esRutaExe(ruta),
// mutuamente excluyentes acá igual que en el popup (ver
// comp_popup_abrir_extra.ts).
// ======================================================

const INICIAR_TEXTO: Record<IniciarAbrir, string> = {
  ventana: "Ventana",
  minimizado: "Minimizado",
  maximizado: "Maximizado",
};

export function textoAbrirExtra(
  abrirExtra: AbrirExtraPerfil,
  ruta: string | null,
): string {
  const instancias = abrirExtra.instancias === "unica" ? "Única" : "Múltiple";

  const lineas = [
    `Iniciar: ${INICIAR_TEXTO[abrirExtra.iniciar]}`,
    `Instancias: ${instancias}`,
  ];

  if (esRutaExe(ruta)) {
    lineas.push(
      `Argumento: ${abrirExtra.argumento ? abrirExtra.argumento : "Sin argumento"}`,
    );
  } else {
    lineas.push(
      `Abrir con: ${abrirExtra.abrirCon ? nombreDeRuta(abrirExtra.abrirCon) : "Predeterminado"}`,
    );
  }

  return lineas.join("\n");
}

// ======================================================
// 🏷️ NOMBRE DE ARCHIVO A PARTIR DE UNA RUTA ABSOLUTA
// ------------------------------------------------------
// Compartido entre la columna Acción (botón "Seleccionar...", ver
// comp_popup_abrir_accion.ts) y el popup Extra (botón "Abrir con",
// ver comp_popup_abrir_extra.ts) — ambos necesitan mostrar solo el
// nombre del archivo/programa, nunca la ruta completa (esa queda
// como tooltip).
// ======================================================

export function nombreDeRuta(ruta: string): string {
  return ruta.split(/[\\/]/).pop() || ruta;
}

// ======================================================
// 📝 TEXTO ACCIÓN (columna Acción de la tabla)
// ------------------------------------------------------
// "Seleccionar..." hasta que se elige una ruta. A partir de ahí, el
// nombre del archivo/programa — con el argumento personalizado
// agregado a continuación cuando la ruta ES un .exe (spec: "notepad.exe"
// (se agrega acá --argumento en caso de haberse personalizado)").
// abrirCon (el programa alternativo) NO se muestra acá — solo tiene
// sentido dentro del popup Extra, la columna Acción sigue mostrando
// el archivo/carpeta que se abre.
// ======================================================

export function textoAbrirAccion(
  abrirAccion: AbrirAccionPerfil,
  abrirExtra: AbrirExtraPerfil,
): string {
  if (!abrirAccion.ruta) {
    return "Seleccionar...";
  }

  const nombre = nombreDeRuta(abrirAccion.ruta);

  const argumento = abrirExtra.argumento.trim();

  if (esRutaExe(abrirAccion.ruta) && argumento) {
    return `${nombre} ${argumento}`;
  }

  return nombre;
}
