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
  };
}

// ======================================================
// 📝 TEXTO ACCIÓN (columna Acción de la tabla)
// ------------------------------------------------------
// Default "⚡ Editar MenuExpress" hasta que el menú tenga
// nombre — a partir de ahí, rayo + nombre (ver spec).
// ======================================================

export function textoMenuAccion(menuAccion: MenuAccionPerfil): string {
  return menuAccion.nombre
    ? `⚡ ${menuAccion.nombre}`
    : "⚡ Editar MenuExpress";
}

// ======================================================
// 📝 TEXTO EXTRA (columna Extra de la tabla)
// ------------------------------------------------------
// Resumen corto: Forma elegida, + 🔁/⚡ según Comportamiento
// (Toggle/Efímero) para que se distinga sin abrir el popup.
// ======================================================

export function textoMenuExtra(menuExtra: MenuExpressExtraPerfil): string {
  const forma = menuExtra.forma === "cuadricula" ? "Cuadrícula" : "Radial";

  const comportamiento = menuExtra.comportamiento === "efimero" ? "⚡" : "🔁";

  return `${forma} ${comportamiento}`;
}
