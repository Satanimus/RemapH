# ============================================================

# 🧠 RemapH V3

# 02_Core.md

# ============================================================

## 🎯 Propósito

Este documento define la arquitectura actual de Core.

Core contiene modelos y estructuras compartidas.

No contiene botones.

No contiene ventanas.

No contiene Runtime.

No contiene comunicación directa con hardware.

---

# 📑 Índice

1. Objetivos
2. Responsabilidad de Core
3. Perfil UI
4. ContextoFila
5. Entradas
6. Triggers
7. Captura
8. Reglas de dependencia
9. Buenas prácticas

---

# 🎯 1. Objetivos

Core existe para mantener los modelos comunes fuera de la UI y del Runtime.

Su pregunta principal es:

> ¿Qué información debe ser compartida sin pertenecer a una capa visual o física?

---

# 🏛️ 2. Responsabilidad de Core

Core contiene:

• Modelos de perfil.
• Estado editable de la UI.
• Identidad de filas.
• Entradas canónicas.
• Triggers.
• Captura y análisis de captura.

Core no contiene:

• Botones.
• Ventanas.
• Windows API.
• Interception.
• Cache del Runtime.
• Ejecución física.

---

# 👤 3. Perfil UI

El Perfil UI representa la configuración que el usuario está editando.

El modelo principal es:

```
Perfil
│
├── activo
└── filas[]
```

Cada fila es un:

`FilaPerfil`

La UI modifica este modelo.

Flujo:

```
UI
↓
Perfil UI
```

El Runtime no utiliza directamente este modelo.

La conversión al modelo persistente ocurre en Tauri.

---

# 🪪 4. ContextoFila

Cada fila posee una identidad única.

La identidad se representa mediante:

`ContextoFila`

```
Fila
│
└── id
```

Todos los componentes internos de la fila reciben el mismo ContextoFila.

Ejemplo:

```
Fila
│
├── Estado
├── Tipo
├── Acción
└── Color

Todos reciben:

ContextoFila
```

La identidad pertenece a la fila.

No pertenece a los botones.

Una fila clonada recibe un nuevo ID.

---

# ⌨️ 5. Entradas

`core_entrada.ts` define el modelo canónico de entrada.

La UI puede utilizar nombres propios del DOM.

Core los representa en el idioma canónico de RemapH.

Ejemplo:

```
ControlLeft
↓
LeftControl
```

Una entrada contiene:

• Tipo.
• Código.
• Nombre.

La capa superior no debe depender directamente de Windows o Interception.

---

# 🎯 6. Triggers

`core_trigger.ts` define el modelo de Trigger.

```
Trigger
├── modificadores[]
├── gatillo
└── condicion
```

El Trigger también contiene funciones de representación visual:

• Texto.
• HTML.

La representación visual se centraliza en Core para mantener una única interpretación del modelo.

---

# 🎥 7. Captura

La captura se divide en dos etapas.

## Captura

`comp_capturador_captura.ts`

Recibe eventos del DOM.

Construye un timeline.

Convierte entradas al idioma canónico.

---

## Análisis

`core_analizar_captura.ts`

Recibe el timeline.

Analiza la secuencia.

Produce un `Trigger`.

Flujo:

```
DOM
↓
Entrada canónica
↓
EventoCaptura
↓
Timeline
↓
Analizador
↓
Trigger
```

---

# 🔗 8. Reglas de dependencia

Las dependencias permitidas son:

```
UI
↓
Core
↓
Runtime
↓
Platform
```

Core no conoce:

• UI visual.
• Runtime.
• Windows.
• Interception.

---

# ✅ 9. Buenas prácticas

Antes de crear un nuevo modelo en Core:

¿Es compartido?

¿Tiene sentido fuera de la UI?

¿Debe ser independiente del hardware?

¿Pertenece a la configuración o a la ejecución?

Si representa una estructura compilada o una acción física, probablemente no pertenece a Core.

---

# 📌 Resumen

Core mantiene los modelos comunes de RemapH.

Sus responsabilidades actuales son:

• Perfil editable.
• Contexto de fila.
• Entradas.
• Triggers.
• Captura.

Core define información.

No controla la apariencia.

No ejecuta remapeos.
