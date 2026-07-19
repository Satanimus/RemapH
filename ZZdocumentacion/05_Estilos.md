# ============================================================

# 🎨 RemapH V3

# 05_Estilos.md

# ============================================================

## 🎯 Propósito

Este documento define la arquitectura visual de RemapH V3.

Los estilos controlan la apariencia.

No contienen lógica.

No controlan el comportamiento de la aplicación.

---

# 📑 Índice

1. Objetivo visual
2. Organización CSS
3. Variables globales
4. Componentes visuales
5. Filosofía de diseño
6. Reglas de uso

---

# 🎯 1. Objetivo visual

RemapH debe transmitir la apariencia de una herramienta profesional.

La interfaz prioriza:

• Claridad.
• Densidad de información.
• Rapidez de uso.
• Consistencia.
• Ausencia de elementos innecesarios.

La interfaz no debe parecer una página web decorativa.

---

# 🗂️ 2. Organización CSS

En el estado actual los estilos se dividen en:

```
src/styles/

styl_variables.css
styl_general.css
styl_layout.css
styl_tabla.css
styl_botones.css
```

Cada archivo agrupa una responsabilidad visual concreta.

---

## styl_variables.css

Identidad visual centralizada.

---

## styl_general.css

Reglas generales de la aplicación.

---

## styl_layout.css

Toolbar.
Layout.
StatusBar.
Popup global.

---

## styl_tabla.css

Cabecera.
Filas.
Celdas.
Columnas.
Divisores.

---

## styl_botones.css

Botones UI.
Controles de captura.
Elementos visuales de Trigger.

---

# 🎨 3. Variables globales

`styl_variables.css` es la fuente central de identidad visual.

Contiene:

• Tipografía.
• Fondos.
• Bordes.
• Texto.
• Colores.
• Colores de filas.
• Radios.
• Espaciados.
• Alturas.
• Anchos de columnas.
• Velocidad de animación.

Los demás archivos deben utilizar estas variables cuando exista una variable adecuada.

---

# 🧩 4. Componentes visuales

La UI se organiza visualmente mediante:

```
Toolbar
↓
Layout

Tabla
↓
Cabecera + Filas

Botones
↓
Controles UI
```

Los estilos deben mantenerse separados por responsabilidad.

---

# 🧠 5. Filosofía de diseño

## Minimalismo funcional

Cada elemento debe existir porque cumple una función.

## Información primero

La interfaz debe favorecer:

• Lectura rápida.
• Identificación de estados.
• Configuración eficiente.

## Densidad controlada

La tabla debe mostrar mucha información sin convertirse en una interfaz visualmente caótica.

## Sin recursos externos

No se utilizan paquetes de iconos ni assets decorativos externos.

Se prioriza:

• CSS.
• Tipografía.
• Unicode.

---

# 📏 6. Reglas de uso

## No repetir valores

Si un color o tamaño existe como variable, debe utilizarse la variable.

## No mezclar estructura y apariencia

TypeScript define qué existe.

CSS define cómo se ve.

## Estilos inline

Los estilos inline deben utilizarse únicamente cuando una propiedad depende directamente de un valor dinámico.

Ejemplo:

Ancho de columna.

## Mantener coherencia visual

Un nuevo control debe reutilizar:

• Colores existentes.
• Radios existentes.
• Tamaños existentes.
• Estados existentes.

---

# ✅ Resumen

Los estilos de RemapH son una capa visual independiente.

Sus reglas principales:

• Variables centralizadas.
• CSS separado por responsabilidad.
• Sin recursos externos innecesarios.
• Diseño profesional.
• Información primero.
