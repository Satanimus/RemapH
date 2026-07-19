# ============================================================

# 📌 RemapH V3

# 07_Pendientes.md

# ============================================================

## 🎯 Propósito

Este documento registra funcionalidades, mejoras e investigaciones pendientes del proyecto.

No contiene decisiones definitivas de arquitectura.

No sustituye los documentos oficiales de cada capa.

Su objetivo es mantener una lista clara de trabajo futuro.

---

# 📑 Índice

1. Pendientes de desarrollo
2. Mejoras de arquitectura
3. Sistemas futuros
4. Documentación futura
5. Ideas descartadas o en evaluación

---

# 🔨 1. Pendientes de desarrollo

## 🔵 Captura avanzada

El sistema base de captura ya existe.

Actualmente soporta:

• Teclado.
• Mouse.
• Rueda.
• Combinaciones.
• Modificadores.
• Análisis de timeline.

Pendiente:

• Ampliar la captura a secuencias más complejas.
• Definir estados temporales avanzados.
• Mejorar el modelo de captura para futuros tipos de entrada.

---

## 🔵 Sistema de perfiles

Implementar gestión completa de perfiles.

Debe incluir:

• Crear perfil.
• Seleccionar perfil.
• Renombrar perfil.
• Clonar perfil.
• Eliminar perfil.
• Restaurar perfil.

El flujo de perfiles debe mantenerse separado del Runtime.

El perfil editable debe permanecer en la UI/Core.

El perfil persistente debe utilizar PerfilJson.

---

## 🔵 Preferencias globales

Agregar configuración general:

• Tiempos.
• Comportamientos por defecto.
• Apariencia.
• Opciones del sistema.

---

## 🔵 Editor de macros

Crear editor visual para secuencias complejas.

Debe permanecer separado de la configuración básica de una fila.

---

## 🔵 Ventanas especializadas

Crear ventanas independientes cuando la complejidad lo justifique.

Ejemplos:

• Multimedia avanzada.
• Programas.
• Coordenadas.
• Macros.
• Joystick.

---

# 🏗️ 2. Mejoras de arquitectura

## 🧠 Sistema ADR

Crear un sistema de registro de decisiones arquitectónicas.

Cada decisión importante debe documentar:

• Fecha.
• Problema existente.
• Opciones consideradas.
• Decisión tomada.
• Motivo.
• Consecuencias.

Las decisiones no deben depender únicamente de memoria personal.

---

# 🧩 3. Sistemas futuros

## 🔵 Joystick

Agregar soporte para dispositivos adicionales.

Debe seguir la arquitectura:

```
Platform
↓
InputEvent
↓
Runtime
↓
AccionCache
↓
Platform
```

---

## 🔵 Sistema de conflictos

Detectar:

• Dos remapeos usando el mismo Trigger.
• Acciones incompatibles.
• Configuraciones ambiguas.

---

## 🔵 Importación y exportación

Permitir compartir configuraciones.

Debe mantenerse independiente del modelo compilado.

---

## 🔵 Multimedia

Completar la representación y ejecución de acciones multimedia.

---

# 📚 4. Documentación futura

Mantener actualizados:

• Índice maestro.
• Documentos de arquitectura.
• ADR.
• Notas de versiones.

Toda decisión importante debe documentarse antes de convertirse en una dependencia del proyecto.

---

# 💭 5. Ideas descartadas o en evaluación

Esta sección almacena ideas que todavía no tienen una decisión definitiva.

Una idea pendiente no debe convertirse automáticamente en una implementación.

Primero debe evaluarse su impacto arquitectónico.

---

# ✅ Resumen

Los pendientes representan el futuro del proyecto.

Sus reglas principales:

• Separar ideas de decisiones.
• Registrar cambios importantes.
• Mantener la arquitectura antes de agregar funciones.
• Resolver complejidad con módulos nuevos cuando corresponda.
• Evitar crecimiento desordenado.
