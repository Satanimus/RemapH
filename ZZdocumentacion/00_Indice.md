# ============================================================

# 📚 RemapH V3

# 00_Indice.md

# ============================================================

## 🎯 Propósito

Este documento es el punto de entrada de la documentación oficial de RemapH V3.

No describe el funcionamiento interno de una capa específica.

Su objetivo es mostrar la estructura general del proyecto y servir como mapa para localizar rápidamente cualquier parte de la documentación.

Toda decisión arquitectónica importante debe quedar documentada en alguno de los documentos indicados aquí.

---

# 📑 Índice

1. Objetivos del proyecto
2. Arquitectura general
3. Organización de la documentación
4. Organización del código
5. Estado del proyecto
6. Convenciones generales
7. Última actualización

---

# 🎯 1. Objetivos del proyecto

RemapH V3 es un remapeador de dispositivos de entrada para Windows.

Su objetivo es traducir entradas físicas en acciones configurables por el usuario mediante una arquitectura modular.

Los pilares del proyecto son:

• Simplicidad.
• Escalabilidad.
• Alto rendimiento.
• Arquitectura desacoplada.
• Mantenimiento sencillo.

Toda decisión futura deberá respetar estos principios.

---

# 🏛️ 2. Arquitectura general

El proyecto está dividido en cuatro capas conceptuales.

```
UI
↓
Core
↓
Runtime
↓
Platform
```

Cada capa posee una responsabilidad diferente.

Las capas superiores pueden utilizar las inferiores.

Las capas inferiores no conocen detalles internos de las superiores.

---

## 🟦 UI

Responsable de la interfaz y la edición visual del perfil.

Incluye:

• Toolbar.
• Tabla.
• Filas.
• Controles.
• Popups.
• Captura visual.

No ejecuta remapeos.

---

## 🟨 Core

Responsable de los modelos y estructuras compartidas.

Incluye:

• Perfil UI.
• Contexto de fila.
• Entradas.
• Triggers.
• Captura y análisis de captura.

No conoce Windows.

---

## 🟩 Runtime

Responsable de resolver remapeos.

Incluye:

• Compilador.
• Perfil compilado.
• Cache.
• Motor de ejecución.

No conoce la UI.

---

## 🟥 Platform

Responsable de la comunicación con Windows y hardware.

Incluye:

• Interception.
• Hooks de Windows.
• SendInput.
• Captura física.
• Salida física.

No decide qué remapeo debe ejecutarse.

---

# 📂 3. Organización de la documentación

```
ZZdocumentacion/

000_Arbol_Archivos.md
00_Indice.md
01_UI.md
02_Core.md
03_Runtime.md
04_Platform.md
05_Estilos.md
06_Principios.md
07_Pendientes.md
```

Cada documento responde una pregunta concreta.

| Documento          | Contenido                                                      |
| ------------------ | -------------------------------------------------------------- |
| 000_Arbol_Archivos | Estructura real del proyecto y responsabilidad de cada módulo. |
| 00_Indice          | Visión general.                                                |
| 01_UI              | Arquitectura de la interfaz.                                   |
| 02_Core            | Modelos y estructuras compartidas.                             |
| 03_Runtime         | Compilación, Cache y ejecución.                                |
| 04_Platform        | Integración física con Windows.                                |
| 05_Estilos         | Identidad visual.                                              |
| 06_Principios      | Reglas arquitectónicas permanentes.                            |

---

# 📁 4. Organización del código

```
TypeScript
├── UI
└── Core

Rust
├── Runtime
└── Platform
```

La separación por lenguaje es una decisión técnica.

La separación por responsabilidad es una decisión arquitectónica.

---

# 🟢 5. Estado del proyecto

| Área          | Estado             |
| ------------- | ------------------ |
| UI            | 🟡 En construcción |
| Core          | 🟢 Base funcional  |
| Runtime       | 🟢 Base funcional  |
| Platform      | 🟢 Base funcional  |
| Estilos       | 🟡 En crecimiento  |
| Documentación | 🟡 En construcción |

El proyecto se encuentra en una etapa de construcción de la base arquitectónica y funcional.

---

# 📐 6. Convenciones generales

Todo archivo debe tener una única responsabilidad.

Toda carpeta debe agrupar responsabilidades relacionadas.

Antes de crear un nuevo módulo debe responderse:

> ¿A qué capa pertenece?

Si la respuesta no es evidente, probablemente la responsabilidad todavía no está bien definida.
