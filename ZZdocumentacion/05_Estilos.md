# ============================================================

# 🎨 RemapH V3

# 05_Estilos.md

# ============================================================

## 🎯 Propósito

Este documento define la arquitectura visual de RemapH V3.
Describe cómo deben organizarse los estilos, dónde viven las decisiones visuales y qué reglas deben seguir todos los componentes de interfaz.
Los estilos controlan la apariencia.
No contienen lógica.
No controlan comportamiento.

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
La interfaz debe priorizar:
• Claridad.
• Densidad de información.
• Rapidez de uso.
• Consistencia.
• Ausencia de elementos innecesarios.

La interfaz no debe parecer:
• Página web.
• Aplicación decorativa.
• Panel lleno de efectos.

---

# 🗂️ 2. Organización CSS

Los estilos se dividen por responsabilidad.

```
styles/
├── variables.css
├── general.css
├── layout.css
├── toolbar.css
├── tabla.css
├── fila.css
├── botones.css
├── statusbar.css
├── formularios.css
├── menus.css
├── ventanas.css
├── animaciones.css
└── iconos.css
```

Cada archivo debe contener únicamente estilos de su área.
No duplicar reglas entre archivos.

---

# 🎨 3. Variables globales

`variables.css` es la fuente única de identidad visual.
Aquí viven:
• Colores.
• Tipografías.
• Tamaños.
• Espaciados.
• Radios.
• Medidas comunes.
• Duraciones de animación.

Ejemplo conceptual:
```
--color-fondo
--color-acento
--color-error
--altura-fila
--espaciado-base
```

Los demás archivos CSS deben utilizar variables.
No deben repetir valores directamente.

---

# 🧩 4. Componentes visuales

Cada componente visual tiene su propio CSS cuando la complejidad lo justifica.
Ejemplos:

```
Toolbar
↓
toolbar.css

Tabla
↓
tabla.css

Botones
↓
botones.css
```

Un componente no debe modificar visualmente otro componente.

---

# 🧠 5. Filosofía de diseño

La identidad visual de RemapH se basa en:

## Minimalismo funcional
Cada elemento debe existir porque tiene una función.

## Sin recursos externos
No utilizar:
• Imágenes decorativas.
• Paquetes de iconos.
• Assets gráficos externos.

Utilizar preferentemente:
• CSS.
• Tipografía.
• Unicode cuando sea suficiente.
• SVG propios cuando sean necesarios.

## Información primero
La interfaz debe favorecer:
• Lectura rápida.
• Identificación de estados.
• Configuración eficiente.

---

# 📏 6. Reglas de uso

## No repetir valores
Si un color o tamaño existe como variable, debe utilizarse esa variable.

## No mezclar estructura y apariencia
HTML/TypeScript define:
• Qué existe.
CSS define:
• Cómo se ve.

## No usar estilos inline salvo casos excepcionales
Los estilos deben permanecer centralizados.

## Mantener coherencia visual
Un botón nuevo debe reutilizar:
• tamaños existentes.
• colores existentes.
• estados existentes.
• animaciones existentes.

---

# ✅ Resumen

Los estilos de RemapH son una capa visual independiente.
Sus reglas principales:
• Variables primero.
• Componentes separados.
• Sin recursos externos.
• Sin duplicación.
• Diseño profesional y funcional.