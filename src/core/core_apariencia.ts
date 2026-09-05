// ======================================================
// 🎨 core_Apariencia
// ------------------------------------------------------
// Aplica los overrides CSS guardados (pestaña Apariencia de
// la Ventana de Configuración) como estilo inline en <html>,
// apenas arranca cada ventana — así todas parten ya con la
// paleta/tamaños personalizados, sin depender de un segundo
// paso posterior al primer pintado.
//
// Lo importan main.ts, captura_main.ts, menu_express_main.ts,
// portapapeles_main.ts y configuracion_main.ts: cada uno lo
// llama una sola vez, apenas se ejecuta el módulo. Cuando se
// guarda un cambio de Apariencia (o se carga un tema), el
// backend recarga (location.reload) todas las ventanas
// abiertas — ver comandos.rs::configuracion_refrescar_ventanas_apariencia —
// y, al recargar, este módulo vuelve a correr y aplica los
// valores nuevos.
//
// El estilo inline en <html> siempre gana por especificidad
// sobre las reglas ":root { ... }" de styl_variables.css (sin
// importar en qué orden se importen los CSS), así que no hace
// falta ninguna coordinación extra con ese archivo.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

// Claves "--xxx" aplicadas la última vez que corrió esta función en
// ESTA ventana. Las demás ventanas no lo necesitan (se recargan
// enteras vía location.reload y arrancan con <html> limpio), pero la
// propia Ventana de Configuración llama esta función de nuevo sin
// recargar (ver vent_configuracion_main.ts::refrescarTrasCambioApariencia)
// — sin este registro, una clave que tenía valor en la llamada
// anterior y ya no viene en el mapa nuevo (cambio de tema con menos
// overrides, restablecer, etc.) se quedaba pisando el valor viejo.
let clavesAplicadasPrevias: string[] = [];

// Convierte un color hex (#rrggbb) a rgba(...) con el alpha dado —
// usado para los "blobs" decorativos de fondo-general (ver más
// abajo), que antes eran rgba(0,212,255,.15)/rgba(13,110,253,.15)
// fijos en styl_layout.css sin relación con fondo-general-color1/2.
function hexARgba(hex: string, alpha: number): string {
  const limpio = hex.replace("#", "");

  const bigint = Number.parseInt(limpio, 16);

  const r = (bigint >> 16) & 255;
  const g = (bigint >> 8) & 255;
  const b = bigint & 255;

  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

export async function aplicarOverridesApariencia(): Promise<void> {
  try {
    const overrides = await invoke<Record<string, string>>(
      "obtener_overrides_apariencia",
    );

    const raiz = document.documentElement;

    const clavesNuevas = new Set(
      Object.keys(overrides).map((clave) => `--${clave}`),
    );

    for (const clave of clavesAplicadasPrevias) {
      if (!clavesNuevas.has(clave)) {
        raiz.style.removeProperty(clave);
      }
    }

    for (const [clave, valor] of Object.entries(overrides)) {
      raiz.style.setProperty(`--${clave}`, valor);
    }

    clavesAplicadasPrevias = [...clavesNuevas];

    // [FIX] Antes .app/.layout (styl_layout.css) traían dos
    // radial-gradient fijos (cyan/azul de marca) apilados ENCIMA de
    // var(--fondo-general), sin relación con fondo-general-color1/2:
    // en modo "Plano" seguían viéndose esos dos blobs en las
    // esquinas (con solo el centro en el color plano elegido), y en
    // cualquier modo no reaccionaban a un cambio de Color 1/Color 2.
    // --fondo-general se arma acá completo (blobs incluidos) según
    // el modo, y se usa igual en las 3 ventanas (html,body en
    // styl_general.css; .app/.layout en styl_layout.css): en "Plano"
    // es el color 1 solo, sin blobs; en "Degradado" son los blobs
    // (coloreados con color 1/color 2 reales, no fijos) más el
    // degradado lineal de base.
    const modo = overrides["fondo-general-modo"] ?? "degradado";
    const color1 = overrides["fondo-general-color1"] ?? "#00292e";
    const color2 = overrides["fondo-general-color2"] ?? "#000924";

    raiz.style.setProperty(
      "--fondo-general",
      modo === "degradado"
        ? `radial-gradient(at 0% 0%, ${hexARgba(color1, 0.15)} 0px, transparent 50%), ` +
            `radial-gradient(at 100% 100%, ${hexARgba(color2, 0.15)} 0px, transparent 50%), ` +
            "linear-gradient(180deg, var(--fondo-general-color1), var(--fondo-general-color2))"
        : "var(--fondo-general-color1)",
    );
  } catch (error) {
    console.error(
      "⚠️ No se pudieron aplicar los overrides de Apariencia:",
      error,
    );
  }
}
