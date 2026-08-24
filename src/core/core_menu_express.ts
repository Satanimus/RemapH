// ======================================================
// ⚡ core_Menu_Express
// ------------------------------------------------------
// Modelo del tipo "MenuExpress" (filaPerfil.tipo === "menu_express").
// El id de la propia fila ES el id del menú — no se genera ni
// guarda ningún identificador aparte.
//
// Accion (menuAccion) y Extra (menuExtra) son dos objetos siempre
// presentes en la fila, igual que coordenada — solo tienen efecto
// cuando tipo === "menu_express". compilador.rs (Rust) los empaqueta
// juntos en una sola AccionCache::MenuExpress al compilar: Runtime
// recibe todo de una — nombre, botones, forma, comportamiento,
// ubicación y tamaños — y se lo pasa entero a back_menu_express.rs
// para dibujar la ventana. No hay un ExtraCache separado para este
// tipo (evita pisar el mecanismo genérico de Turbo/Mantener).
//
// Espejo exacto de MenuAccionJson / MenuExpressExtraJson en
// perfil_json.rs (mismos nombres de campo, camelCase) — viaja tal
// cual hacia Rust en compilar_perfil, sin traducción adicional.
// ======================================================

export type FormaMenu = "radial" | "cuadricula";

export type ComportamientoMenu = "toggle" | "efimero";

export type UbicacionMenu = "persistente" | "cursor";

export type TamanoMenu = "pequeno" | "mediano" | "grande";

// Nueva variable global de popup Extra (pulido, punto "Color
// botón"): Monocromo (default) deja los botones como estaban antes
// — heredan el color de fondo de la ventana (menuExtra.color, la
// FILA MenuExpress). Color le da a cada botón el borde del color de
// SU PROPIA fila referenciada (ver MenuBotonPerfil.filaId más
// abajo) — resuelto del lado de Rust (compilador.rs), no acá.
export type ColorBotonMenu = "color" | "monocromo";

// ======================================================
// 🔘 BOTÓN DEL MENÚ
// ------------------------------------------------------
// filaId es el id INTERNO de la fila referenciada (no su número de
// orden en la tabla). renombrar es editable por el usuario en el
// editor — arranca con el texto que muestra la columna Acción de
// esa fila al momento de agregarla (ver etapa del editor).
// ======================================================

export interface MenuBotonPerfil {
  filaId: string;

  renombrar: string;
}

// ======================================================
// 🎯 ACCIÓN DEL MENÚ (columna Acción)
// ======================================================

export interface MenuAccionPerfil {
  nombre: string;

  botones: MenuBotonPerfil[];
}

// ======================================================
// 🎛️ EXTRA DEL MENÚ (columna Extra)
// ------------------------------------------------------
// columnas/filas: 0 significa "Auto" (se acomoda al número de
// atajos) — solo uno de los dos puede ser distinto de 0 a la vez;
// la UI impone esa regla (ver la etapa del popup Extra). Ambos
// solo tienen sentido cuando forma === "cuadricula".
// ======================================================

export interface MenuExpressExtraPerfil {
  forma: FormaMenu;

  columnas: number;

  filas: number;

  comportamiento: ComportamientoMenu;

  ubicacion: UbicacionMenu;

  tamanoBoton: TamanoMenu;

  tamanoTexto: TamanoMenu;

  colorBoton: ColorBotonMenu;
}

// ======================================================
// ➕ CREAR ACCIÓN / EXTRA POR DEFECTO
// ======================================================

export function crearMenuAccion(): MenuAccionPerfil {
  return {
    nombre: "",

    botones: [],
  };
}

export function crearMenuExtra(): MenuExpressExtraPerfil {
  return {
    forma: "radial",

    columnas: 0,

    filas: 2,

    comportamiento: "toggle",

    ubicacion: "persistente",

    tamanoBoton: "mediano",

    tamanoTexto: "mediano",

    colorBoton: "monocromo",
  };
}

// ======================================================
// 📝 TEXTO ACCIÓN (columna Acción de la tabla)
// ------------------------------------------------------
// Default "Editar" hasta que el menú tenga nombre — a partir
// de ahí, solo el nombre (sin ícono: los íconos de columna
// Acción se sacaron de la ventana principal, quedan solo en
// el popup de Tipo, ver comp_popup_abrir.ts).
// ======================================================

export function textoMenuAccion(menuAccion: MenuAccionPerfil): string {
  return menuAccion.nombre ? menuAccion.nombre : "Editar";
}

// ======================================================
// 📝 TOOLTIP DEL BOTÓN EXTRA (columna Extra, ícono ⁘)
// ------------------------------------------------------
// Lista de líneas "Subtítulo: Elección" — todos los campos de
// menuExtra. Columnas/Filas solo si forma === "cuadricula".
// "Botones" combina tamaño+color en una línea (mismo criterio
// que la fila fusionada del popup, ver
// comp_popup_menu_express_extra.ts::crearFilaBotones).
// ======================================================

function textoTamanoMenu(tamano: TamanoMenu): string {
  switch (tamano) {
    case "pequeno":
      return "Pequeño";

    case "mediano":
      return "Mediano";

    case "grande":
      return "Grande";
  }
}

export function textoMenuExtra(menuExtra: MenuExpressExtraPerfil): string {
  const forma = menuExtra.forma === "cuadricula" ? "Cuadrícula" : "Radial";

  const lineas = [`Forma: ${forma}`];

  if (menuExtra.forma === "cuadricula") {
    lineas.push(
      `Columnas: ${menuExtra.columnas === 0 ? "Auto" : menuExtra.columnas}`,
    );

    lineas.push(`Filas: ${menuExtra.filas === 0 ? "Auto" : menuExtra.filas}`);
  }

  lineas.push(
    `Comportamiento: ${menuExtra.comportamiento === "efimero" ? "Efímero" : "Toggle"}`,
  );

  lineas.push(
    `Ubicación: ${menuExtra.ubicacion === "cursor" ? "Cursor" : "Persistente"}`,
  );

  const color = menuExtra.colorBoton === "color" ? "Color" : "Monocromo";

  lineas.push(`Botones: ${textoTamanoMenu(menuExtra.tamanoBoton)} ${color}`);

  lineas.push(`Texto: ${textoTamanoMenu(menuExtra.tamanoTexto)}`);

  return lineas.join("\n");
}
